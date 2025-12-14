use std::convert::Infallible;

use uom::si::f64::{MassDensity, ThermodynamicTemperature};

use crate::{State, fluid::Stateless};

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
    type Error: std::error::Error + Send + Sync + 'static;

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
