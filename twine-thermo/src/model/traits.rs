use std::convert::Infallible;

use uom::si::f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature};

use crate::{
    PropertyError, State,
    fluid::Stateless,
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

/// Trait for creating thermodynamic states from various input combinations.
///
/// This trait allows models to create a `State<Fluid>` from different types
/// of thermodynamic inputs, offering flexibility in how states are specified.
///
/// Common input patterns for [`Stateless`] fluids include tuples such as:
/// - `(ThermodynamicTemperature, MassDensity)` - Direct temperature and density
/// - `(ThermodynamicTemperature, Pressure)` - Temperature and pressure (model calculates density)
/// - `(Pressure, MassDensity)` - Pressure and density (model calculates temperature)
/// - `(Pressure, SpecificEntropy)` - Pressure and entropy (model calculates temperature and density)
///
/// Single values are also supported, such as `ThermodynamicTemperature` alone for
/// incompressible [`Stateless`] fluids, which use the fluid's reference density.
///
/// Models implement this trait for specific `Input` types to indicate which
/// combinations they support.
///
/// A blanket implementation is provided for all [`Stateless`] fluids using
/// `(ThermodynamicTemperature, MassDensity)` as the input.
pub trait StateFrom<Fluid, Input> {
    type Error;

    /// Returns a `State<Fluid>` based on the provided generic `Input`.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the state cannot be created from the given inputs.
    /// The error type depends on the specific model implementation.
    fn state_from(&self, input: Input) -> Result<State<Fluid>, Self::Error>;
}

/// Enables state creation from temperature and density for any [`Stateless`] fluid.
impl<Model, Fluid: Stateless> StateFrom<Fluid, (ThermodynamicTemperature, MassDensity)> for Model {
    type Error = Infallible;

    fn state_from(
        &self,
        (temperature, density): (ThermodynamicTemperature, MassDensity),
    ) -> Result<State<Fluid>, Self::Error> {
        Ok(State {
            temperature,
            density,
            fluid: Fluid::default(),
        })
    }
}
