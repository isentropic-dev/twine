use uom::si::f64::{SpecificHeatCapacity, ThermalConductivity};

use super::DensityModel;

/// Fluid properties used by the stratified tank.
#[derive(Debug, Clone)]
pub struct Fluid<D: DensityModel> {
    /// Provides density as a function of temperature.
    pub density_model: D,
    /// Constant specific heat capacity of the fluid.
    pub specific_heat: SpecificHeatCapacity,
    /// Constant thermal conductivity.
    pub thermal_conductivity: ThermalConductivity,
}
