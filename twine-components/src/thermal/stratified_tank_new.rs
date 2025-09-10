mod buoyancy;
mod energy_balance;
mod environment;
mod fluid;
mod geometry;
mod insulation;
mod location;
mod mass_balance;
mod node;
mod port_flow;

use std::{array, ops::Div};

use thiserror::Error;
use twine_core::TimeDerivative;
use twine_thermo::HeatFlow;
use uom::{
    ConstZero,
    si::f64::{
        HeatCapacity, Ratio, ThermalConductance, ThermodynamicTemperature, Volume, VolumeRate,
    },
};

use node::{Adjacent, Node};

pub use environment::Environment;
pub use fluid::Fluid;
pub use geometry::Geometry;
pub use insulation::Insulation;
pub use location::{Location, PortLocation};
pub use port_flow::PortFlow;

/// A stratified thermal energy storage tank.
///
/// Represents a fixed-geometry tank with `N` fully mixed vertical nodes.
/// Supports energy exchange through `P` port pairs and `Q` auxiliary heat sources.
///
/// A **port pair** models a real-world connection between the tank and an
/// external hydraulic circuit:
/// - One end returns fluid to the tank at a known temperature.
/// - The other end draws fluid out of the tank at the same volumetric rate.
///
/// This pairing maintains mass balance in the tank.
/// The outlet temperature for the port pair comes from the node(s) where the
/// outflow is taken.
///
/// The location of ports and auxiliary heat sources are fixed when the tank is
/// created; each may apply to a single node or be split across multiple nodes.
///
/// Generic over:
/// - `N`: number of nodes
/// - `P`: number of port pairs
/// - `Q`: number of auxiliary heat sources
#[derive(Debug)]
pub struct StratifiedTank<const N: usize, const P: usize, const Q: usize> {
    nodes: [Node<P, Q>; N],
    vols: [Volume; N],
}

/// Input to the stratified tank component.
///
/// Captures the runtime state needed to evaluate the tank's thermal response.
/// Locations of ports and auxiliary heat sources, and how they are distributed
/// across nodes, are fixed when the tank is created.
/// This struct holds only the values that change at runtime.
///
/// Generic over:
/// - `N`: Number of nodes
/// - `P`: Number of port pairs
/// - `Q`: Number of auxiliary heat sources
pub struct Input<const N: usize, const P: usize, const Q: usize> {
    /// Temperatures of the `N` nodes, from bottom to top.
    ///
    /// Values do not need to be thermally stable; if warmer nodes appear below
    /// cooler ones, the model mixes them before computing energy balances.
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
/// Captures the thermally stable node temperatures and their time derivatives
/// after applying mass and energy balances.
///
/// Generic over:
/// - `N`: Number of nodes
#[derive(Debug, Clone)]
pub struct Output<const N: usize> {
    /// Temperatures of the `N` nodes, from bottom to top.
    ///
    /// These values are guaranteed to be thermally stable.
    pub temperatures: [ThermodynamicTemperature; N],

    /// Time derivatives of temperature in each node.
    ///
    /// Includes the effects of fluid flow, auxiliary heat input, conduction to
    /// neighboring nodes, and environmental heat loss or gain.
    pub derivatives: [TimeDerivative<ThermodynamicTemperature>; N],
}

/// Errors that can occur when creating a [`StratifiedTank`].
#[derive(Debug, Error)]
pub enum StratifiedTankCreationError {
    #[error("geometry is invalid: {0}")]
    Geometry(String),
    #[error("aux[{index}] location is invalid: {context}")]
    AuxLocation { index: usize, context: String },
    #[error("port[{index}] inlet location is invalid: {context}")]
    PortInletLocation { index: usize, context: String },
    #[error("port[{index}] outlet location is invalid: {context}")]
    PortOutletLocation { index: usize, context: String },
}

impl<const P: usize, const Q: usize> StratifiedTank<0, P, Q> {
    /// Creates a new stratified tank.
    ///
    /// Specify the number of nodes (`N`) with a const generic parameter on `new()`.
    /// For example, to create a tank with 20 nodes:
    ///
    /// ```ignore
    /// let tank = StratifiedTank::new::<20>(fluid, geometry, insulation, aux, ports)?;
    /// ```
    ///
    /// The number of port pairs (`P`) and auxiliary heat sources (`Q`) are inferred
    /// from the lengths of `port_locations` and `aux_locations`, respectively.
    ///
    /// # Parameters
    ///
    /// - `fluid`: Incompressible [`Fluid`] properties.
    /// - `geometry`: [`Geometry`] used to derive per-node volumes and areas.
    /// - `insulation`: [`Insulation`] at the tank surfaces.
    /// - `aux_locations`: Placement of each auxiliary heat source as a [`Location`].
    /// - `port_locations`: Placement of each port pair's inlet and outlet as a [`PortLocation`].
    ///
    /// # Errors
    ///
    /// Returns a [`StratifiedTankCreationError`] if the tank cannot be created.
    ///
    /// # Example
    ///
    /// The following code creates a perfectly insulated cylindrical tank with a
    /// 10-inch auxiliary heater and a bottom-inlet/top-outlet port pair.
    ///
    /// ```
    /// use twine_components::thermal::stratified_tank_new::{
    ///     Fluid, Geometry, Insulation, Location, PortLocation, StratifiedTank,
    /// };
    /// use uom::si::{
    ///     f64::{
    ///         Length, MassDensity, SpecificHeatCapacity, ThermalConductivity,
    ///         ThermodynamicTemperature,
    ///     },
    ///     length::{inch, meter},
    ///     mass_density::kilogram_per_cubic_meter,
    ///     specific_heat_capacity::kilojoule_per_kilogram_kelvin,
    ///     thermal_conductivity::watt_per_meter_kelvin,
    /// };
    ///
    /// let fluid = Fluid {
    ///     density: MassDensity::new::<kilogram_per_cubic_meter>(1000.0),
    ///     specific_heat: SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.186),
    ///     thermal_conductivity: ThermalConductivity::new::<watt_per_meter_kelvin>(0.6),
    /// };
    ///
    /// let geometry = Geometry::VerticalCylinder {
    ///     diameter: Length::new::<meter>(0.5),
    ///     height: Length::new::<meter>(1.8),
    /// };
    ///
    /// let insulation = Insulation::Adiabatic;
    ///
    /// let el_center = Length::new::<meter>(1.5);
    /// let el_length = Length::new::<inch>(10.0);
    /// let aux_locations = [Location::span_abs(el_center, el_length)];
    ///
    /// let port_locations = [PortLocation {
    ///     inlet: Location::tank_bottom(),
    ///     outlet: Location::tank_top(),
    /// }];
    ///
    /// // Create a tank with 5 nodes.
    /// let tank = StratifiedTank::new::<5>(fluid, geometry, insulation, aux_locations, port_locations)
    ///     .expect("valid configuration");
    /// ```
    pub fn new<const N: usize>(
        fluid: Fluid,
        geometry: Geometry,
        insulation: Insulation,
        aux_locations: [Location; Q],
        port_locations: [PortLocation; P],
    ) -> Result<StratifiedTank<N, P, Q>, StratifiedTankCreationError> {
        let node_geometries = geometry
            .into_node_geometries::<N>()
            .map_err(StratifiedTankCreationError::Geometry)?;

        let heights = node_geometries.map(|node| node.height);

        // Calculate aux weights.
        let mut aux_weight_by_node = [[0.0; Q]; N];
        for (index, loc) in aux_locations.iter().enumerate() {
            let weights = loc
                .into_weights(&heights)
                .map_err(|context| StratifiedTankCreationError::AuxLocation { index, context })?;
            for node_idx in 0..N {
                aux_weight_by_node[node_idx][index] = weights[node_idx];
            }
        }

        // Calculate port weights.
        let mut inlet_weight_by_node = [[0.0; P]; N];
        let mut outlet_weight_by_node = [[0.0; P]; N];
        for (index, port_loc) in port_locations.iter().enumerate() {
            let inlet_weights = port_loc.inlet.into_weights(&heights).map_err(|context| {
                StratifiedTankCreationError::PortInletLocation { index, context }
            })?;
            let outlet_weights = port_loc.outlet.into_weights(&heights).map_err(|context| {
                StratifiedTankCreationError::PortOutletLocation { index, context }
            })?;
            for node_idx in 0..N {
                inlet_weight_by_node[node_idx][index] = inlet_weights[node_idx];
                outlet_weight_by_node[node_idx][index] = outlet_weights[node_idx];
            }
        }

        let mut nodes = array::from_fn(|i| {
            let node = node_geometries[i];
            let aux_heat_weights = aux_weight_by_node[i];
            let port_inlet_weights = inlet_weight_by_node[i];
            let port_outlet_weights = outlet_weight_by_node[i];

            Node {
                inv_volume: node.volume.recip(),
                inv_heat_capacity: (node.volume * fluid.density * fluid.specific_heat).recip(),
                // TODO: The bottom and top UA values are calculated assuming nodes have equal area.
                //       I think we'll need to scale them by adjacent area ratios if they aren't.
                ua: Adjacent {
                    bottom: node.height * fluid.thermal_conductivity,
                    side: match insulation {
                        Insulation::Adiabatic => ThermalConductance::ZERO,
                    },
                    top: node.height * fluid.thermal_conductivity,
                },
                aux_heat_weights,
                port_inlet_weights,
                port_outlet_weights,
            }
        });

        // Fix UA values for bottom and top nodes.
        nodes[0].ua.bottom = match insulation {
            Insulation::Adiabatic => ThermalConductance::ZERO,
        };
        nodes[N - 1].ua.top = match insulation {
            Insulation::Adiabatic => ThermalConductance::ZERO,
        };

        Ok(StratifiedTank {
            nodes,
            vols: node_geometries.map(|node| node.volume),
        })
    }
}

impl<const N: usize, const P: usize, const Q: usize> StratifiedTank<N, P, Q> {
    /// Evaluates the tank's thermal response at a single point in time.
    ///
    /// Enforces thermal stability by mixing unstable nodes, then applies mass
    /// and energy balances to determine per-node temperature derivatives.
    #[must_use]
    pub fn call(&self, input: &Input<N, P, Q>) -> Output<N> {
        let Input {
            temperatures: t_guess,
            port_flows,
            aux_heat_flows,
            environment,
        } = input;

        // Stabilize node temperatures.
        let mut temperatures = *t_guess;
        buoyancy::stabilize(&mut temperatures, &self.vols);

        // Compute node-to-node flow.
        let upward_flows = mass_balance::compute_upward_flows(
            &port_flows.map(PortFlow::into_rate),
            &self.nodes.map(|n| n.port_inlet_weights),
            &self.nodes.map(|n| n.port_outlet_weights),
        );

        // Calculate the total derivative for each node: flows + aux + conduction.
        let derivatives = array::from_fn(|i| {
            self.deriv_from_flows(i, &temperatures, &upward_flows, port_flows)
                + self.deriv_from_aux(i, aux_heat_flows)
                + self.deriv_from_conduction(i, &temperatures, environment)
        });

        Output {
            temperatures,
            derivatives,
        }
    }

    /// Computes the `i`th node's `dT/dt` due to fluid flows.
    fn deriv_from_flows(
        &self,
        i: usize,
        temperatures: &[ThermodynamicTemperature; N],
        upward_flows: &[VolumeRate; N],
        port_flows: &[PortFlow; P],
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let node = self.nodes[i];

        // Optional flow from the node below.
        let flow_from_below = if i > 0 && upward_flows[i - 1] > VolumeRate::ZERO {
            Some((upward_flows[i - 1], temperatures[i - 1]))
        } else {
            None
        };

        // Optional flow from the node above.
        let flow_from_above = if i < N - 1 && upward_flows[i] < VolumeRate::ZERO {
            Some((-upward_flows[i], temperatures[i + 1]))
        } else {
            None
        };

        let inflows = port_flows
            .iter()
            .zip(node.port_inlet_weights)
            .map(|(port_flow, weight)| (port_flow.rate() * weight, port_flow.inlet_temperature))
            .chain(flow_from_below)
            .chain(flow_from_above);

        energy_balance::derivative_from_fluid_flows(temperatures[i], node.inv_volume, inflows)
    }

    /// Computes the `i`th node's `dT/dt` due to auxiliary heat sources.
    fn deriv_from_aux(
        &self,
        i: usize,
        aux_heat_flows: &[HeatFlow; Q],
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let node = self.nodes[i];

        let heat_flows = aux_heat_flows
            .iter()
            .zip(node.aux_heat_weights)
            .map(|(q_dot, weight)| q_dot.signed() * weight);

        energy_balance::derivative_from_heat_flows(node.inv_heat_capacity, heat_flows)
    }

    /// Computes the `i`th node's `dT/dt` due to conduction to its surroundings.
    fn deriv_from_conduction(
        &self,
        i: usize,
        temperatures: &[ThermodynamicTemperature; N],
        env: &Environment,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let node = self.nodes[i];

        let t_bottom = if i == 0 {
            env.bottom
        } else {
            temperatures[i - 1]
        };

        let t_top = if i == N - 1 {
            env.top
        } else {
            temperatures[i + 1]
        };

        energy_balance::deriv_from_conduction(
            temperatures[i],
            Adjacent {
                bottom: t_bottom,
                side: env.side,
                top: t_top,
            },
            node.ua,
            node.inv_heat_capacity,
        )
    }
}

type InverseHeatCapacity = <Ratio as Div<HeatCapacity>>::Output;
type InverseVolume = <Ratio as Div<Volume>>::Output;

#[cfg(test)]
mod tests {
    use super::*;

    use std::f64::consts::PI;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{
            Length, MassDensity, Power, SpecificHeatCapacity, ThermalConductivity,
            ThermodynamicTemperature, VolumeRate,
        },
        length::meter,
        mass_density::kilogram_per_cubic_meter,
        power::kilowatt,
        specific_heat_capacity::kilojoule_per_kilogram_kelvin,
        thermodynamic_temperature::degree_celsius,
        volume_rate::gallon_per_minute,
    };

    // Test tank:
    // - vertical cylinder
    // - 3 nodes, each V=1 m³ and UA=0
    // - 1 port: inlet 100% to node 0, outlet 100% from node 2
    // - 1 aux: 100% applied to node 2
    fn test_tank() -> StratifiedTank<3, 1, 1> {
        let fluid = Fluid {
            density: MassDensity::new::<kilogram_per_cubic_meter>(1000.0),
            specific_heat: SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.0),
            thermal_conductivity: ThermalConductivity::ZERO,
        };

        let geometry = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>((4.0 / PI).sqrt()),
            height: Length::new::<meter>(3.0),
        };

        let insulation = Insulation::Adiabatic;

        let aux_locations = [Location::tank_top()];

        let port_locations = [PortLocation {
            inlet: Location::tank_bottom(),
            outlet: Location::tank_top(),
        }];

        StratifiedTank::new(fluid, geometry, insulation, aux_locations, port_locations).unwrap()
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
