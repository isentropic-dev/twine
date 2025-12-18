use twine_core::TimeIntegrable;
use uom::si::{
    f64::{MassDensity, SpecificHeatCapacity, Time},
    mass_density::kilogram_per_cubic_meter,
    specific_heat_capacity::kilojoule_per_kilogram_kelvin,
};

use crate::model::incompressible::{IncompressibleFluid, IncompressibleParameters};

/// Marker type for liquid water.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Water;

impl IncompressibleFluid for Water {
    fn parameters() -> IncompressibleParameters {
        IncompressibleParameters::new(
            SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.184),
            MassDensity::new::<kilogram_per_cubic_meter>(997.047),
        )
    }
}

impl TimeIntegrable for Water {
    type Derivative = ();

    fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
        self
    }
}
