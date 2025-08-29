//! Stratified thermal storage tank component.
//!
//! Models a tank divided into `N` fully mixed vertical layers,
//! with energy and mass exchange via configured inlet ports,
//! auxiliary heat sources, and ambient boundary conditions.

#![allow(dead_code)]

mod adjacent;
mod buoyancy;
mod environment;
mod layer;
mod mass_balance;
mod port_flow;

use std::array;

use twine_core::TimeDerivative;
use twine_thermo::HeatFlow;
use uom::{
    ConstZero,
    si::f64::{
        MassDensity, SpecificHeatCapacity, ThermalConductance, ThermodynamicTemperature, Volume,
        VolumeRate,
    },
};

use adjacent::Adjacent;
use layer::Layer;

pub use environment::Environment;
pub use port_flow::PortFlow;

pub trait DensityModel {
    fn density(&self, temperature: ThermodynamicTemperature) -> MassDensity;
}

/// A stratified thermal energy storage tank.
///
/// Represents a fixed-geometry tank with `N` fully mixed vertical layers.
/// Supports energy exchange through `P` port pairs and `Q` auxiliary heat sources.
///
/// A **port pair** models a real-world connection between the tank and an
/// external hydraulic circuit:
/// - One end returns fluid to the tank at a known temperature.
/// - The other end draws fluid out of the tank at the same volumetric rate.
///
/// This pairing maintains mass balance in the tank.
/// The outlet temperature for the port pair comes from the layer(s) where the
/// outflow is taken.
///
/// The location of port pairs and auxiliary heat sources are fixed when the
/// tank is constructed; each may apply to a single layer or be split across
/// multiple layers.
///
/// Generic over:
/// - `D`: density model that provides `rho = f(T)`
/// - `N`: number of layers
/// - `P`: number of port pairs
/// - `Q`: number of auxiliary heat sources
pub struct StratifiedTank<D: DensityModel, const N: usize, const P: usize, const Q: usize> {
    cp: SpecificHeatCapacity,
    dens_model: D,
    nodes: [Node<P, Q>; N],
}

/// Input to the stratified tank component.
///
/// Captures the runtime state needed to evaluate the tank's thermal response.
/// Locations of ports and auxiliary heat sources, and how they are distributed
/// across layers, are fixed when the tank is constructed.
/// This struct holds only the values that change at runtime.
///
/// Generic over:
/// - `N`: Number of stratified layers
/// - `P`: Number of port pairs
/// - `Q`: Number of auxiliary heat sources
pub struct Input<const N: usize, const P: usize, const Q: usize> {
    /// Temperatures of the `N` layers, from bottom to top.
    ///
    /// Each layer is fully mixed.
    /// Values do not need to be stratified; if warmer layers appear below
    /// cooler ones, the model mixes adjacent layers before computing energy balances.
    pub temperatures: [ThermodynamicTemperature; N],

    /// Flow rates and temperatures for the `P` port pairs.
    pub port_flows: [PortFlow; P],

    /// Heat input or extraction for the `Q` auxiliary sources.
    pub aux_heat_flows: [HeatFlow; Q],

    /// Ambient temperatures outside the tank.
    pub environment: Environment,
}

/// Output from the stratified tank component.
///
/// Captures the thermally stable layer temperatures and their time derivatives
/// after applying mass and energy balances.
///
/// Generic over:
/// - `N`: Number of stratified layers
pub struct Output<const N: usize> {
    /// Temperatures of the `N` layers, from bottom to top.
    ///
    /// These values are guaranteed to be thermally stable.
    pub temperatures: [ThermodynamicTemperature; N],

    /// Time derivatives of temperature in each layer.
    ///
    /// Includes the effects of mass flow, auxiliary heat input, and
    /// environmental heat loss or gain.
    pub derivatives: [TimeDerivative<ThermodynamicTemperature>; N],
}

impl<D: DensityModel, const N: usize, const P: usize, const Q: usize> StratifiedTank<D, N, P, Q> {
    /// Evaluates the tank's thermal response at a single point in time.
    ///
    /// Enforces thermal stability by mixing unstable layers, then applies port
    /// flows, auxiliary heat input, and environmental effects.
    #[must_use]
    pub fn call(&self, input: &Input<N, P, Q>) -> Output<N> {
        let Input {
            temperatures: t_guess,
            port_flows,
            aux_heat_flows,
            environment,
        } = input;

        // Create stabilized layers.
        let vol = array::from_fn(|i| self.nodes[i].vol);
        let layers = buoyancy::stabilize(t_guess, &vol, &self.dens_model)
            .map(|(temp, mass)| Layer::new(temp, mass, self.cp));

        // Compute layer-to-layer flow.
        let upward_flows = mass_balance::compute_upward_flows(
            &port_flows.map(PortFlow::into_rate),
            &array::from_fn(|i| self.nodes[i].port_inlet_weights),
            &array::from_fn(|i| self.nodes[i].port_outlet_weights),
        );

        // Calculate the total derivative for each layer: flows + aux + conduction.
        let derivatives = array::from_fn(|i| {
            self.deriv_from_flows(i, &layers, &upward_flows, port_flows)
                + self.deriv_from_aux(i, &layers, aux_heat_flows)
                + self.deriv_from_conduction(i, &layers, environment)
        });

        Output {
            temperatures: layers.map(Layer::into_temperature),
            derivatives,
        }
    }

    /// Computes the `i`th layer's temperature derivative due to fluid flows.
    fn deriv_from_flows(
        &self,
        i: usize,
        layers: &[Layer; N],
        upward_flows: &[VolumeRate; N],
        port_flows: &[PortFlow; P],
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let layer = layers[i];

        // Optional flow from the layer below.
        let flow_from_below = if i > 0 && upward_flows[i - 1] > VolumeRate::ZERO {
            Some((upward_flows[i - 1], layers[i - 1].temp))
        } else {
            None
        };

        // Optional flow from the layer above.
        let flow_from_above = if i < N - 1 && upward_flows[i] < VolumeRate::ZERO {
            Some((-upward_flows[i], layers[i + 1].temp))
        } else {
            None
        };

        let inflows = port_flows
            .iter()
            .zip(self.nodes[i].port_inlet_weights)
            .map(|(port_flow, weight)| (port_flow.rate() * weight, port_flow.inlet_temperature))
            .chain(flow_from_below)
            .chain(flow_from_above)
            .map(|(v_dot, temp)| (v_dot * self.dens_model.density(temp), temp));

        layer.derivative_from_fluid_flows(inflows)
    }

    /// Computes the `i`th layer's temperature derivative due to auxiliary heat sources.
    fn deriv_from_aux(
        &self,
        i: usize,
        layers: &[Layer; N],
        aux_heat_flows: &[HeatFlow; Q],
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let layer = layers[i];

        // Weight each aux source by this layer's allocation.
        let heat_flows = aux_heat_flows
            .iter()
            .zip(self.nodes[i].aux_heat_weights)
            .map(|(q_dot, weight)| q_dot.signed() * weight);

        layer.derivative_from_heat_flows(heat_flows)
    }

    /// Computes the `i`th layer's temperature derivative due to conduction to surroundings.
    fn deriv_from_conduction(
        &self,
        i: usize,
        layers: &[Layer; N],
        env: &Environment,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let layer = layers[i];

        let bottom = if i == 0 {
            env.bottom
        } else {
            layers[i - 1].temp
        };

        let top = if i == N - 1 {
            env.top
        } else {
            layers[i + 1].temp
        };

        layer.derivative_from_conduction(
            self.nodes[i].ua,
            Adjacent {
                bottom,
                side: env.side,
                top,
            },
        )
    }
}

struct Node<const P: usize, const Q: usize> {
    vol: Volume,
    ua: Adjacent<ThermalConductance>,
    aux_heat_weights: [f64; Q],
    port_inlet_weights: [f64; P],
    port_outlet_weights: [f64; P],
}

#[cfg(test)]
mod tests {
    use super::{Adjacent, *};

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, Power, ThermodynamicTemperature, Volume, VolumeRate},
        mass_density::kilogram_per_cubic_meter,
        power::kilowatt,
        specific_heat_capacity::kilojoule_per_kilogram_kelvin,
        thermodynamic_temperature::degree_celsius,
        volume::cubic_meter as m3,
        volume_rate::gallon_per_minute,
    };

    struct ConstantDensity {
        density: MassDensity,
    }

    impl ConstantDensity {
        fn new(rho_kg_m3: f64) -> Self {
            Self {
                density: MassDensity::new::<kilogram_per_cubic_meter>(rho_kg_m3),
            }
        }
    }

    impl DensityModel for ConstantDensity {
        fn density(&self, _temperature: ThermodynamicTemperature) -> MassDensity {
            self.density
        }
    }

    // Test tank:
    // - 3 layers, each V=1 m³ and UA=0
    // - 1 port: inlet 100% to layer 0, outlet 100% from layer 2
    // - 1 aux: 100% applied to layer 2
    fn test_tank() -> StratifiedTank<ConstantDensity, 3, 1, 1> {
        let bottom = Node {
            vol: Volume::new::<m3>(1.0),
            ua: Adjacent::default(),
            aux_heat_weights: [0.0],
            port_inlet_weights: [1.0],
            port_outlet_weights: [0.0],
        };

        let middle = Node {
            vol: Volume::new::<m3>(1.0),
            ua: Adjacent::default(),
            aux_heat_weights: [0.0],
            port_inlet_weights: [0.0],
            port_outlet_weights: [0.0],
        };

        let top = Node {
            vol: Volume::new::<m3>(1.0),
            ua: Adjacent::default(),
            aux_heat_weights: [1.0],
            port_inlet_weights: [0.0],
            port_outlet_weights: [1.0],
        };

        StratifiedTank {
            dens_model: ConstantDensity::new(1000.0),
            cp: SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.0),
            nodes: [bottom, middle, top],
        }
    }

    fn port_flow(gpm: f64, celsius: f64) -> PortFlow {
        PortFlow::new(
            VolumeRate::new::<gallon_per_minute>(gpm),
            ThermodynamicTemperature::new::<degree_celsius>(celsius),
        )
        .unwrap()
    }

    fn zero_port_flows<const P: usize>() -> [PortFlow; P] {
        [port_flow(0.0, 25.0); P]
    }

    #[test]
    fn at_equilibrium_all_zero_derivatives() {
        let tank = test_tank();
        let t = ThermodynamicTemperature::new::<degree_celsius>(20.0);

        let input = Input {
            temperatures: [t; 3],
            port_flows: zero_port_flows(),
            aux_heat_flows: [HeatFlow::None],
            environment: Environment {
                bottom: t,
                side: t,
                top: t,
            },
        };

        let out = tank.call(&input);

        // Temps unchanged after buoyancy (already monotonic).
        assert!(out.temperatures.iter().all(|&node_temp| node_temp == t));

        // All derivatives should be zero.
        for temp_derivative in out.derivatives {
            assert_relative_eq!(temp_derivative.value, 0.0);
        }
    }

    #[test]
    fn aux_only_heats_target_node() {
        let tank = test_tank();
        let t = ThermodynamicTemperature::new::<degree_celsius>(50.0);

        let input = Input {
            temperatures: [t; 3],
            port_flows: zero_port_flows(),
            aux_heat_flows: [HeatFlow::from_signed(Power::new::<kilowatt>(20.0)).unwrap()],
            environment: Environment {
                bottom: t,
                side: t,
                top: t,
            },
        };

        let out = tank.call(&input);

        // Q/C = Q / (V * rho * cp)
        //     = 20 kW / (1 m³ * 1,000 kg/m³ * 4 kJ/(kg·K))
        //     = 20,000 / 4,000,000 = 0.005 K/s at layer 2
        assert_relative_eq!(out.derivatives[0].value, 0.0);
        assert_relative_eq!(out.derivatives[1].value, 0.0);
        assert_relative_eq!(out.derivatives[2].value, 0.005);
    }
}
