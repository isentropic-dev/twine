use uom::si::f64::{SpecificHeatCapacity, ThermalConductivity};

use super::DensityModel;

#[derive(Debug, Clone)]
pub struct Fluid<D: DensityModel> {
    pub density_model: D,
    pub specific_heat: SpecificHeatCapacity,
    pub thermal_conductivity: ThermalConductivity,
}
