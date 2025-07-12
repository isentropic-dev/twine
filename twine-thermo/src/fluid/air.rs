use uom::si::{
    f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
    pressure::atmosphere,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::degree_celsius,
};

use crate::{model::ideal_gas::IdealGasFluid, units::SpecificGasConstant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Air;

/// Standard ideal gas properties for dry air.
///
/// TODO: Find a standard to reference and double check these values.
impl IdealGasFluid for Air {
    fn gas_constant(&self) -> SpecificGasConstant {
        SpecificGasConstant::new::<joule_per_kilogram_kelvin>(287.053)
    }

    fn cp(&self) -> SpecificHeatCapacity {
        SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1005.0)
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<degree_celsius>(0.0)
    }

    fn reference_pressure(&self) -> Pressure {
        Pressure::new::<atmosphere>(1.0)
    }
}
