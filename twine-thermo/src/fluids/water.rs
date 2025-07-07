use twine_core::TimeIntegrable;
use uom::si::{
    f64::{SpecificHeatCapacity, ThermodynamicTemperature, Time},
    specific_heat_capacity::kilojoule_per_kilogram_kelvin,
    thermodynamic_temperature::degree_celsius,
};

use crate::IncompressibleProperties;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Water;

/// Standard properties for incompressible water.
///
/// TODO: Find a standard to reference and double check these values.
impl IncompressibleProperties for Water {
    fn specific_heat(&self) -> SpecificHeatCapacity {
        SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.184)
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<degree_celsius>(25.0)
    }
}

impl TimeIntegrable for Water {
    type Derivative = ();

    fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
        self
    }
}
