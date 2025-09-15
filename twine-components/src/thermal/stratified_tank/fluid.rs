use uom::si::f64::{MassDensity, SpecificHeatCapacity, ThermalConductivity};

/// Constant fluid properties used by the stratified tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fluid {
    pub density: MassDensity,
    pub specific_heat: SpecificHeatCapacity,
    pub thermal_conductivity: ThermalConductivity,
}
