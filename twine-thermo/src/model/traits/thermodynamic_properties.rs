use uom::si::f64::{Pressure, SpecificHeatCapacity};

use crate::{
    PropertyError, State,
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy},
};

/// Trait for computing thermodynamic properties from a fluid's state.
///
/// Provides methods for retrieving pressure, internal energy, enthalpy, entropy,
/// and specific heats from a [`State`] parameterized over a fluid type.
///
/// Models can implement this trait in two ways, depending on their scope:
///
/// - A general-purpose model can work with any fluid implementing a capability trait.
///   For example, [`IdealGas`] works with any fluid that implements [`IdealGasFluid`].
///
/// - Alternatively, a model can be implemented for a specific fluid type,
///   such as `impl ThermodynamicProperties<Water> for MyEmpiricalWaterModel`,
///   to support only that fluid with tighter domain control.
///
/// This approach supports both broad reuse and precise specialization,
/// depending on the modeling context.
pub trait ThermodynamicProperties<Fluid> {
    /// Returns the pressure for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the pressure cannot be calculated.
    fn pressure(&self, state: &State<Fluid>) -> Result<Pressure, PropertyError>;

    /// Returns the specific internal energy for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the internal energy cannot be calculated.
    fn internal_energy(
        &self,
        state: &State<Fluid>,
    ) -> Result<SpecificInternalEnergy, PropertyError>;

    /// Returns the specific enthalpy for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the enthalpy cannot be calculated.
    fn enthalpy(&self, state: &State<Fluid>) -> Result<SpecificEnthalpy, PropertyError>;

    /// Returns the specific entropy for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the entropy cannot be calculated.
    fn entropy(&self, state: &State<Fluid>) -> Result<SpecificEntropy, PropertyError>;

    /// Returns the specific heat capacity at constant pressure.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if `cp` cannot be calculated.
    fn cp(&self, state: &State<Fluid>) -> Result<SpecificHeatCapacity, PropertyError>;

    /// Returns the specific heat capacity at constant volume.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if `cv` cannot be calculated.
    fn cv(&self, state: &State<Fluid>) -> Result<SpecificHeatCapacity, PropertyError>;
}
