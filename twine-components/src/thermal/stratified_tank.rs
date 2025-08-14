//! Stratified thermal storage tank component.
//!
//! Models a tank divided into `N` fully mixed vertical layers,
//! with energy and mass exchange via configured inlet ports,
//! auxiliary heat sources, and ambient boundary conditions.

#![allow(dead_code)]

mod buoyancy;
mod environment;
mod mass_balance;
mod node;
mod port_flow;

use std::array;

use twine_thermo::{HeatFlow, model::incompressible::IncompressibleFluid};
use uom::{
    ConstZero,
    si::f64::{ThermodynamicTemperature, VolumeRate},
};

use buoyancy::{Layer, apply_buoyancy};
use mass_balance::compute_upward_flows;
use node::{Node, NodeTemperatures};
use twine_core::TimeDerivative;

pub use environment::Environment;
pub use port_flow::PortFlow;

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
/// - `F`: fluid type (must implement [`IncompressibleFluid`])
/// - `N`: number of layers
/// - `P`: number of port pairs
/// - `Q`: number of auxiliary heat sources
pub struct StratifiedTank<F: IncompressibleFluid, const N: usize, const P: usize, const Q: usize> {
    fluid: F,
    nodes: [Node; N],
    aux_heat_weights: [[f64; Q]; N],
    port_inlet_weights: [[f64; P]; N],
    port_outlet_weights: [[f64; P]; N],
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

impl<F: IncompressibleFluid, const N: usize, const P: usize, const Q: usize>
    StratifiedTank<F, N, P, Q>
{
    /// Evaluates the tank's thermal response at a single point in time.
    ///
    /// Enforces thermal stability by mixing unstable layers, then applies port
    /// flows, auxiliary heat input, and environmental effects.
    pub fn call(&self, input: &Input<N, P, Q>) -> Output<N> {
        let Input {
            temperatures,
            port_flows,
            aux_heat_flows,
            environment,
        } = input;

        // Apply buoyancy-driven mixing.
        let temperatures: [ThermodynamicTemperature; N] = apply_buoyancy(array::from_fn(|i| {
            Layer::new(temperatures[i], self.nodes[i].volume)
        }));

        // Compute node-to-node flow.
        let upward_flows = compute_upward_flows(
            &array::from_fn(|k| port_flows[k].rate.into_inner()),
            &self.port_inlet_weights,
            &self.port_outlet_weights,
        );

        // Calculate the total derivative for each node: flows + aux + conduction.
        let derivatives: [TimeDerivative<ThermodynamicTemperature>; N] = array::from_fn(|i| {
            self.deriv_from_flows(i, &temperatures, &upward_flows, port_flows)
                + self.deriv_from_aux(i, aux_heat_flows)
                + self.deriv_from_conduction(i, &temperatures, environment)
        });

        Output {
            temperatures,
            derivatives,
        }
    }

    /// Computes the node's temperature derivative from all fluid inflows.
    fn deriv_from_flows(
        &self,
        i: usize,
        temps: &[ThermodynamicTemperature; N],
        upward_flows: &[VolumeRate; N],
        port_flows: &[PortFlow; P],
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let node = self.nodes[i];
        let t_node = temps[i];

        // Optional flow from the node below.
        let flow_from_below = if i > 0 && upward_flows[i - 1] > VolumeRate::ZERO {
            Some((upward_flows[i - 1], temps[i - 1]))
        } else {
            None
        };

        // Optional flow from the node above.
        let flow_from_above = if i < N - 1 && upward_flows[i] < VolumeRate::ZERO {
            Some((-upward_flows[i], temps[i + 1]))
        } else {
            None
        };

        let inflows = port_flows
            .iter()
            .zip(self.port_inlet_weights[i])
            .map(|(port_flow, weight)| {
                (
                    port_flow.rate.into_inner() * weight,
                    port_flow.inlet_temperature,
                )
            })
            .chain(flow_from_below)
            .chain(flow_from_above);

        node.derivative_from_fluid_flows(t_node, inflows)
    }

    /// Computes the node's temperature derivative from auxiliary heat sources.
    fn deriv_from_aux(
        &self,
        i: usize,
        aux_heat_flows: &[HeatFlow; Q],
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let node = self.nodes[i];

        // Weight each aux source by this node's allocation.
        let heat_flows = aux_heat_flows
            .iter()
            .zip(self.aux_heat_weights[i])
            .map(|(q_dot, weight)| q_dot.signed() * weight);

        node.derivative_from_heat_flows(heat_flows)
    }

    /// Computes the node's temperature derivative from conduction to surroundings.
    fn deriv_from_conduction(
        &self,
        i: usize,
        temps: &[ThermodynamicTemperature; N],
        env: &Environment,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let node = self.nodes[i];

        let bottom = if i == 0 { env.bottom } else { temps[i - 1] };
        let top = if i == N - 1 { env.top } else { temps[i + 1] };

        node.derivative_from_conduction(NodeTemperatures {
            center: temps[i],
            bottom,
            side: env.side,
            top,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{
            MassDensity, Power, SpecificHeatCapacity, ThermalConductance, ThermodynamicTemperature,
            Volume, VolumeRate,
        },
        mass_density::kilogram_per_cubic_meter,
        power::kilowatt,
        specific_heat_capacity::kilojoule_per_kilogram_kelvin,
        thermodynamic_temperature::degree_celsius,
        volume::cubic_meter as m3,
        volume_rate::gallon_per_minute,
    };

    struct TestFluid;
    impl IncompressibleFluid for TestFluid {
        fn specific_heat(&self) -> SpecificHeatCapacity {
            SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.0)
        }
        fn reference_temperature(&self) -> ThermodynamicTemperature {
            ThermodynamicTemperature::new::<degree_celsius>(25.0)
        }
        fn reference_density(&self) -> MassDensity {
            MassDensity::new::<kilogram_per_cubic_meter>(1000.0)
        }
    }

    // Test tank:
    // - 3 nodes, each V=1 m³ and UA=0
    // - 1 port: inlet 100% to node 0, outlet 100% from node 2
    // - 1 aux: 100% applied to node 2
    fn test_tank() -> StratifiedTank<TestFluid, 3, 1, 1> {
        let volume = Volume::new::<m3>(1.0);
        let heat_capacity = volume * TestFluid.reference_density() * TestFluid.specific_heat();

        let nodes = [Node::new(
            volume,
            heat_capacity,
            ThermalConductance::ZERO,
            ThermalConductance::ZERO,
            ThermalConductance::ZERO,
        ); 3];

        let aux_heat_weights = [[0.0], [0.0], [1.0]]; // all aux on top node
        let port_inlet_weights = [[1.0], [0.0], [0.0]]; // inlet to bottom node
        let port_outlet_weights = [[0.0], [0.0], [1.0]]; // outlet from top node

        StratifiedTank {
            fluid: TestFluid,
            nodes,
            aux_heat_weights,
            port_inlet_weights,
            port_outlet_weights,
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
        //     = 20,000 / 4,000,000 = 0.005 K/s at node 2
        assert_relative_eq!(out.derivatives[0].value, 0.0);
        assert_relative_eq!(out.derivatives[1].value, 0.0);
        assert_relative_eq!(out.derivatives[2].value, 0.005);
    }
}
