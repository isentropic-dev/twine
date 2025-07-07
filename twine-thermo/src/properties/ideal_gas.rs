use uom::{
    ConstZero,
    si::f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
};

use crate::units::{SpecificEnthalpy, SpecificEntropy, SpecificGasConstant};

/// Trait used to define thermodynamic constants for ideal gases.
///
/// This trait provides the fixed properties required to model a fluid using
/// ideal gas assumptions, such as the specific gas constant `R`, constant
/// pressure heat capacity `cp`, and reference conditions.
///
/// Typically implemented for simple fluids like [`Air`] or [`CarbonDioxide`],
/// this trait enables reuse across models that support ideal gases,
/// such as the [`IdealGas`] model.
///
/// You can also implement this trait for custom fluids to extend the framework:
///
/// ```ignore
/// use twine_thermo::{IdealGasProperties, units::SpecificGasConstant};
/// use uom::si::f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// struct MyGas;
///
/// impl IdealGasProperties for MyGas {
///     fn gas_constant(&self) -> SpecificGasConstant { /* ... */ }
///     fn cp(&self) -> SpecificHeatCapacity { /* ... */ }
///     fn reference_temperature(&self) -> ThermodynamicTemperature { /* ... */ }
///     fn reference_pressure(&self) -> Pressure { /* ... */ }
/// }
/// ```
pub trait IdealGasProperties {
    /// Returns the specific gas constant `R`.
    fn gas_constant(&self) -> SpecificGasConstant;

    /// Returns the specific heat capacity at constant pressure `cp`.
    fn cp(&self) -> SpecificHeatCapacity;

    /// Returns the reference temperature used in enthalpy and entropy calculations.
    fn reference_temperature(&self) -> ThermodynamicTemperature;

    /// Returns the reference pressure used in entropy calculations.
    fn reference_pressure(&self) -> Pressure;

    /// Returns the enthalpy at the reference temperature.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_enthalpy(&self) -> SpecificEnthalpy {
        SpecificEnthalpy::ZERO
    }

    /// Returns the entropy at the reference temperature and pressure.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_entropy(&self) -> SpecificEntropy {
        SpecificEntropy::ZERO
    }
}
