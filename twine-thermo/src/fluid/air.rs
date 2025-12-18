use twine_core::TimeIntegrable;
use uom::si::{
    f64::{SpecificHeatCapacity, Time},
    specific_heat_capacity::joule_per_kilogram_kelvin,
};

use crate::{
    model::perfect_gas::{PerfectGasFluid, PerfectGasParameters},
    units::SpecificGasConstant,
};

/// Marker type for dry air.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Air;

impl TimeIntegrable for Air {
    type Derivative = ();

    fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
        self
    }
}

impl PerfectGasFluid for Air {
    fn parameters() -> PerfectGasParameters {
        PerfectGasParameters::new(
            SpecificGasConstant::new::<joule_per_kilogram_kelvin>(287.053),
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1005.0),
        )
    }
}
