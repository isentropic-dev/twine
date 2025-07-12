use std::convert::Infallible;

use uom::si::f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature};

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
pub trait ThermodynamicProperties<F> {
    /// Returns the pressure for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the pressure cannot be calculated.
    fn pressure(&self, state: &State<F>) -> Result<Pressure, PropertyError>;

    /// Returns the specific internal energy for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the internal energy cannot be calculated.
    fn internal_energy(&self, state: &State<F>) -> Result<SpecificInternalEnergy, PropertyError>;

    /// Returns the specific enthalpy for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the enthalpy cannot be calculated.
    fn enthalpy(&self, state: &State<F>) -> Result<SpecificEnthalpy, PropertyError>;

    /// Returns the specific entropy for the given state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the entropy cannot be calculated.
    fn entropy(&self, state: &State<F>) -> Result<SpecificEntropy, PropertyError>;

    /// Returns the specific heat capacity at constant pressure.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if `cp` cannot be calculated.
    fn cp(&self, state: &State<F>) -> Result<SpecificHeatCapacity, PropertyError>;

    /// Returns the specific heat capacity at constant volume.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if `cv` cannot be calculated.
    fn cv(&self, state: &State<F>) -> Result<SpecificHeatCapacity, PropertyError>;
}

/// Trait for creating thermodynamic states from various input combinations.
///
/// This trait enables models to construct a `State<F>` from different types of
/// thermodynamic inputs, providing flexibility in how states are specified.
///
/// This trait is commonly implemented for fluid types that have `Default`,
/// allowing the model to create the fluid instance internally from just
/// thermodynamic properties.
///
/// Common input patterns include tuples of thermodynamic properties:
/// - `(ThermodynamicTemperature, MassDensity)` - Direct temperature and density
/// - `(ThermodynamicTemperature, Pressure)` - Temperature and pressure (model calculates density)
/// - `(Pressure, MassDensity)` - Pressure and density (model calculates temperature)
/// - `(Pressure, SpecificEntropy)` - Pressure and entropy (model calculates temperature and density)
///
/// Single values are also supported, such as `ThermodynamicTemperature` alone
/// for incompressible fluids (which use the fluid's reference density).
///
/// The generic design allows models to define which input combinations they
/// support by implementing this trait for specific `Input` types.
///
/// A blanket implementation is provided for `(ThermodynamicTemperature, MassDensity)`
/// when the fluid type implements `Default`.
pub trait StateFrom<F, Input> {
    type Error;

    /// Returns a `State<F>` based on the provided generic `Input`.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the state cannot be created from the given inputs.
    /// The error type depends on the specific model implementation.
    fn state_from(&self, input: Input) -> Result<State<F>, Self::Error>;
}

/// Enables creating states from temperature and density pairs for any fluid with Default.
impl<Model, Fluid: Default> StateFrom<Fluid, (ThermodynamicTemperature, MassDensity)> for Model {
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
