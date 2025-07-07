use uom::{
    ConstZero,
    si::f64::{SpecificHeatCapacity, ThermodynamicTemperature},
};

use crate::units::{SpecificEnthalpy, SpecificEntropy};

/// Trait used to define thermodynamic constants for incompressible fluids.
///
/// This trait provides the fixed properties needed to model a fluid under
/// incompressible assumptions, such as constant specific heat and a reference
/// temperature for enthalpy and entropy calculations.
///
/// Typically implemented for fluids like [`Water`] or custom liquids, this
/// trait enables models like [`Incompressible`] to compute thermodynamic
/// properties where pressure and density are treated as independent.
///
/// You can also implement this trait for user-defined fluids to support
/// application-specific modeling:
///
/// ```ignore
/// use twine_thermo::IncompressibleProperties;
/// use uom::si::f64::{SpecificHeatCapacity, ThermodynamicTemperature};
///
/// struct MyLiquid;
///
/// impl IncompressibleProperties for MyLiquid {
///     fn specific_heat(&self) -> SpecificHeatCapacity { /* ... */ }
///     fn reference_temperature(&self) -> ThermodynamicTemperature { /* ... */ }
/// }
/// ```
pub trait IncompressibleProperties {
    /// Returns the specific heat capacity.
    fn specific_heat(&self) -> SpecificHeatCapacity;

    /// Returns the reference temperature used in enthalpy and entropy calculations.
    fn reference_temperature(&self) -> ThermodynamicTemperature;

    /// Returns the enthalpy at the reference temperature.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_enthalpy(&self) -> SpecificEnthalpy {
        SpecificEnthalpy::ZERO
    }

    /// Returns the entropy at the reference temperature.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_entropy(&self) -> SpecificEntropy {
        SpecificEntropy::ZERO
    }
}
