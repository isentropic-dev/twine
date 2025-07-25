use twine_core::TimeIntegrable;
use uom::si::{
    f64::{MassDensity, SpecificHeatCapacity, ThermodynamicTemperature, Time},
    mass_density::kilogram_per_cubic_meter,
    specific_heat_capacity::kilojoule_per_kilogram_kelvin,
    thermodynamic_temperature::degree_celsius,
};

use crate::model::incompressible::IncompressibleFluid;

use super::Stateless;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Water;

impl TimeIntegrable for Water {
    type Derivative = ();

    fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
        self
    }
}

impl Stateless for Water {}

/// Standard properties for incompressible water.
///
/// TODO: Find a standard to reference and double check these values.
impl IncompressibleFluid for Water {
    fn specific_heat(&self) -> SpecificHeatCapacity {
        SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.184)
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<degree_celsius>(25.0)
    }

    fn reference_density(&self) -> MassDensity {
        MassDensity::new::<kilogram_per_cubic_meter>(997.047)
    }
}
