use std::f64::consts::PI;

use uom::{
    ConstZero,
    si::f64::{Length, ThermalConductance},
};

use super::{Adjacent, DensityModel, Fluid, Insulation, Location, Node, PortPairLocation};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Geometry {
    VerticalCylinder { diameter: Length, height: Length },
}

impl Geometry {
    pub(super) fn into_node_array<
        const N: usize,
        const P: usize,
        const Q: usize,
        D: DensityModel,
    >(
        self,
        fluid: &Fluid<D>,
        insulation: Insulation,
        aux_locations: [Location; Q],
        port_locations: [PortPairLocation; P],
    ) -> [Node<P, Q>; N] {
        match self {
            Geometry::VerticalCylinder { diameter, height } => {
                let end_area = 0.25 * PI * diameter * diameter;

                #[allow(clippy::cast_precision_loss)]
                let node_height = height / N as f64;

                let mut nodes = [Node {
                    vol: end_area * node_height,
                    ua: Adjacent {
                        bottom: node_height * fluid.thermal_conductivity,
                        side: match insulation {
                            Insulation::Adiabatic => ThermalConductance::ZERO,
                        },
                        top: node_height * fluid.thermal_conductivity,
                    },
                    aux_heat_weights: aux_locations.map(|l| match l {
                        Location::HeightFraction(constrained) => constrained.into_inner(),
                    }),
                    port_inlet_weights: port_locations.map(|p| match p.inlet {
                        Location::HeightFraction(constrained) => constrained.into_inner(),
                    }),
                    port_outlet_weights: port_locations.map(|p| match p.outlet {
                        Location::HeightFraction(constrained) => constrained.into_inner(),
                    }),
                }; N];

                // Fix ua values for bottom and top nodes.
                nodes[0].ua.bottom = match insulation {
                    Insulation::Adiabatic => ThermalConductance::ZERO,
                };
                nodes[N - 1].ua.top = match insulation {
                    Insulation::Adiabatic => ThermalConductance::ZERO,
                };

                nodes
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, SpecificHeatCapacity, ThermalConductivity, ThermodynamicTemperature},
        length::meter,
        mass_density::kilogram_per_cubic_meter,
        specific_heat_capacity::kilojoule_per_kilogram_kelvin,
        thermal_conductivity::watt_per_meter_kelvin,
        volume::cubic_meter,
    };

    struct ConstantDensity;
    impl DensityModel for ConstantDensity {
        fn density(&self, _temp: ThermodynamicTemperature) -> MassDensity {
            MassDensity::new::<kilogram_per_cubic_meter>(1000.0)
        }
    }

    #[test]
    fn nodes_for_a_simple_vertical_cylinder() {
        let fluid = Fluid {
            density_model: ConstantDensity,
            specific_heat: SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.0),
            thermal_conductivity: ThermalConductivity::new::<watt_per_meter_kelvin>(1.0),
        };
        let geometry = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(1.0 / PI.sqrt()), // results in a top/bottom area of 0.25 m²
            height: Length::new::<meter>(2.0),
        };
        let insulation = Insulation::Adiabatic;
        let aux_sources = [];
        let port_pairs = [];

        let [node] = geometry.into_node_array(&fluid, insulation, aux_sources, port_pairs);

        assert_relative_eq!(node.vol.get::<cubic_meter>(), 0.5);
        assert_eq!(node.ua, Adjacent::default());
        assert_eq!(node.aux_heat_weights.len(), 0);
        assert_eq!(node.port_inlet_weights.len(), 0);
        assert_eq!(node.port_outlet_weights.len(), 0);
    }
}
