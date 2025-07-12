use uom::si::f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature};

use crate::{model::ideal_gas::IdealGasFluid, units::SpecificGasConstant};

/// User-defined ideal gas.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct IdealGasCustom {
    /// Optional name for identification.
    pub name: Option<String>,

    /// Specific gas constant.
    pub specific_gas_constant: SpecificGasConstant,

    /// Specific heat capacity at constant pressure.
    pub cp: SpecificHeatCapacity,

    /// Reference temperature for enthalpy and entropy calculations.
    pub reference_temperature: ThermodynamicTemperature,

    /// Reference pressure for entropy calculations.
    pub reference_pressure: Pressure,
}

impl IdealGasFluid for IdealGasCustom {
    fn gas_constant(&self) -> SpecificGasConstant {
        self.specific_gas_constant
    }
    fn cp(&self) -> SpecificHeatCapacity {
        self.cp
    }
    fn reference_temperature(&self) -> ThermodynamicTemperature {
        self.reference_temperature
    }
    fn reference_pressure(&self) -> Pressure {
        self.reference_pressure
    }
}
