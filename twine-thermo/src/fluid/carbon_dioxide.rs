use twine_core::TimeIntegrable;
use uom::si::{
    f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature, Time},
    pressure::atmosphere,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::degree_celsius,
};

use crate::{model::ideal_gas::IdealGasFluid, units::SpecificGasConstant};

use super::Stateless;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CarbonDioxide;

impl TimeIntegrable for CarbonDioxide {
    type Derivative = ();

    fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
        self
    }
}

impl Stateless for CarbonDioxide {}

/// Standard ideal gas properties for carbon dioxide.
///
/// TODO: Find a standard to reference and double check these values.
impl IdealGasFluid for CarbonDioxide {
    fn gas_constant(&self) -> SpecificGasConstant {
        SpecificGasConstant::new::<joule_per_kilogram_kelvin>(188.92)
    }

    fn cp(&self) -> SpecificHeatCapacity {
        SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(844.0)
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<degree_celsius>(0.0)
    }

    fn reference_pressure(&self) -> Pressure {
        Pressure::new::<atmosphere>(1.0)
    }
}

#[cfg(feature = "coolprop")]
impl crate::model::coolprop::CoolPropFluid for CarbonDioxide {
    const BACKEND: &'static str = "HEOS";
    const NAME: &'static str = "CarbonDioxide";
}
