use twine_core::constraint::{Constrained, ConstraintError, StrictlyPositive};
use uom::si::f64::{MassRate, Power};

use crate::{PropertyError, State, model::ThermodynamicProperties};

/// A stream of fluid at a thermodynamic state.
///
/// A `Stream` represents steady-state transport of mass and energy without storing either.
/// For transient systems with mass or energy storage, use a [`ControlVolume`].
///
/// Zero-flow streams are not physically meaningful.
/// Use `Option<Stream<Fluid>>` to represent an optional or inactive stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stream<Fluid> {
    pub rate: Constrained<MassRate, StrictlyPositive>,
    pub state: State<Fluid>,
}

impl<Fluid> Stream<Fluid> {
    /// Creates a new [`Stream`] from a mass rate and thermodynamic state.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `rate` is not strictly positive.
    pub fn new(rate: MassRate, state: State<Fluid>) -> Result<Self, ConstraintError> {
        let rate = Constrained::new(rate)?;
        Ok(Self::from_constrained(rate, state))
    }

    /// Creates a new [`Stream`] from a pre-validated positive mass rate and state.
    pub fn from_constrained(
        rate: Constrained<MassRate, StrictlyPositive>,
        state: State<Fluid>,
    ) -> Self {
        Self { rate, state }
    }

    /// Returns the enthalpy flow rate of this stream.
    ///
    /// This method computes the energy flow carried by the stream as `ṁ · h`,
    /// where `ṁ` is the mass flow rate and `h` is the enthalpy of its state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if enthalpy cannot be computed.
    pub fn enthalpy_flow<Model>(&self, model: &Model) -> Result<Power, PropertyError>
    where
        Model: ThermodynamicProperties<Fluid>,
    {
        let m_dot = self.rate.into_inner();
        let h = model.enthalpy(&self.state)?;
        Ok(m_dot * h)
    }
}
