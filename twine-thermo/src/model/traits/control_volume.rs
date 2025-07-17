//! Traits for modeling control volumes under different physical assumptions.
//!
//! These traits provide interfaces for transient control volume modeling,
//! supporting a range of boundary conditions and operational constraints
//! common in thermodynamic analysis.
//!
//! # Common Physical Assumptions
//!
//! All traits in this module assume:
//!
//! - The control volume has fixed geometry and size.
//! - The internal fluid state is spatially uniform (well mixed).
//! - Outflow exits at the current internal state.
//! - Heat and work are specified as net energy transfer rates.
//! - Kinetic and potential energy changes are negligible.
//!
//! # Modeling Context
//!
//! In open systems, a control volume's state evolves in response to flows of
//! mass, heat, and work across its boundaries.
//! Mass and energy conservation laws, together with additional physical
//! constraints, determine which variables are solved for in a given scenario:
//!
//! - **Fixed Flow:** All inflow and outflow rates are specified,
//!   and the system's state evolves accordingly.
//! - **Constant Pressure:** One or more system variables are adjusted as needed
//!   to maintain constant pressure.
//! - *(Planned: Constant Density, Variable Volume, etc.)*
//!
//! # Usage
//!
//! Each trait corresponds to a distinct modeling constraint or assumption.
//! Fluid property models implement only those traits that match the scenarios
//! under which they are valid.
//!
//! When modeling a control volume, choose the interface that best reflects your
//! system's physical assumptions and requirements.

use twine_core::{
    TimeDifferentiable,
    constraint::{Constrained, NonNegative, StrictlyPositive},
};
use uom::si::f64::{MassRate, Power, Volume};

use crate::{Flow, PropertyError, State, StateDerivative};

/// A control volume with all flow rates externally specified.
///
/// This trait applies to scenarios where all mass flow rates into and out of
/// the control volume are known.
pub trait ControlVolumeFixedFlow<Fluid: TimeDifferentiable> {
    /// Returns the time derivative of the state.
    ///
    /// # Parameters
    ///
    /// - `volume`: Fixed volume of the control region.
    /// - `state`: Current thermodynamic state of the control volume.
    /// - `inflows`: Incoming flows, each with a mass flow rate and fluid state.
    /// - `outflow`: Specified outflow mass rate.
    /// - `heat_input`: Net heat added to the control volume. Positive when heat enters.
    /// - `power_output`: Net work extracted from the control volume. Positive when work leaves.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the state derivative cannot be computed.
    fn state_derivative_at_fixed_outflow(
        &self,
        volume: Constrained<Volume, StrictlyPositive>,
        state: &State<Fluid>,
        inflows: &[Flow<Fluid>],
        outflow: Constrained<MassRate, NonNegative>,
        heat_input: Power,
        power_output: Power,
    ) -> Result<StateDerivative<Fluid>, PropertyError>;
}

/// A control volume operating at constant pressure.
///
/// This trait applies to scenarios where system variables are adjusted as
/// needed to maintain constant pressure in the control volume.
pub trait ControlVolumeConstantPressure<Fluid: TimeDifferentiable> {
    /// Returns the time derivative of the state and the required outflow mass rate.
    ///
    /// # Parameters
    ///
    /// - `volume`: Fixed volume of the control region.
    /// - `state`: Current thermodynamic state of the control volume.
    /// - `inflows`: Incoming flows, each with a mass flow rate and fluid state.
    /// - `heat_input`: Net heat added to the control volume. Positive when heat enters.
    /// - `power_output`: Net work extracted from the control volume. Positive when work leaves.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the state derivative cannot be computed.
    fn state_derivative_at_constant_pressure(
        &self,
        volume: Constrained<Volume, StrictlyPositive>,
        state: &State<Fluid>,
        inflows: &[Flow<Fluid>],
        heat_input: Power,
        power_output: Power,
    ) -> Result<(StateDerivative<Fluid>, Constrained<MassRate, NonNegative>), PropertyError>;
}
