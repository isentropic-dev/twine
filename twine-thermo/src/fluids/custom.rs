use uom::si::f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature};

use crate::{IdealGasProperties, IncompressibleProperties, units::SpecificGasConstant};

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

impl IdealGasProperties for IdealGasCustom {
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

/// User-defined incompressible fluid.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct IncompressibleCustom {
    /// Optional name for identification.
    pub name: Option<String>,

    /// Specific heat capacity.
    pub specific_heat: SpecificHeatCapacity,

    /// Reference temperature for enthalpy and entropy calculations.
    pub reference_temperature: ThermodynamicTemperature,
}

impl IncompressibleProperties for IncompressibleCustom {
    fn specific_heat(&self) -> SpecificHeatCapacity {
        self.specific_heat
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        self.reference_temperature
    }
}
