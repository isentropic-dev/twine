use twine_core::{
    TimeDifferentiable,
    constraint::{Constrained, NonNegative, StrictlyPositive},
};
use uom::si::f64::{MassRate, Power, Volume};

use crate::{Flow, PropertyError, State, StateDerivative};

/// Trait for modeling the transient dynamics of thermodynamic control volumes.
///
/// This trait defines methods for computing the instantaneous rate of change of
/// a thermodynamic state, based on the conservation of mass and energy within a
/// well-mixed, fixed-volume control region.
///
/// # Physical Assumptions
///
/// - The control volume has fixed geometry and size.
/// - The internal fluid state is spatially uniform.
/// - Outflow exits at the current internal state.
/// - Heat and work are specified as net energy transfer rates.
/// - Kinetic and potential energy changes are negligible.
///
/// # Modeling Constraints
///
/// When the internal state, inflows, and external energy rates are known,
/// conservation of mass and energy yield two equations with three unknowns:
/// `dT/dt`, `dρ/dt`, and `ṁ_out`.
/// This trait includes three methods for this scenario, each introducing a
/// constraint to remove one degree of freedom and fully define the system.
///
/// - [`state_derivative_at_constant_density`] assumes `dρ/dt = 0` and solves
///   for the outflow mass rate `ṁ_out` that maintains constant density.
/// - [`state_derivative_at_constant_pressure`] assumes `dP/dt = 0` and solves
///   for the outflow mass rate `ṁ_out` required to hold pressure constant.
/// - [`state_derivative_at_fixed_outflow`] assumes a known `ṁ_out` and computes
///   the resulting rate of change in density.
///
/// # Future Extensions
///
/// This trait may be extended to support additional constraints or inverse
/// problems, such as solving for heat input given a target pressure trajectory.
pub trait ControlVolumeDynamics<Fluid: TimeDifferentiable> {
    /// Computes the time derivative of a control volume at constant density.
    ///
    /// Uses a variable outflow mass rate to enforce `dρ/dt = 0`, maintaining
    /// constant density while allowing temperature and pressure to change.
    ///
    /// # Parameters
    ///
    /// - `volume`: Fixed volume of the control region.
    /// - `state`: Current thermodynamic state of the control volume.
    /// - `inflows`: Incoming flows, each with a mass flow rate and fluid state.
    /// - `heat_input`: Net heat added to the control volume. Positive when heat enters.
    /// - `power_output`: Net work extracted from the control volume. Positive when work leaves.
    ///
    /// # Returns
    ///
    /// The outflow mass rate and the resulting [`StateDerivative<Fluid>`].
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the state derivative cannot be evaluated.
    fn state_derivative_at_constant_density(
        &self,
        volume: Constrained<Volume, StrictlyPositive>,
        state: &State<Fluid>,
        inflows: &[Flow<Fluid>],
        heat_input: Power,
        power_output: Power,
    ) -> Result<(Constrained<MassRate, NonNegative>, StateDerivative<Fluid>), PropertyError>;

    /// Computes the time derivative of a control volume at constant pressure.
    ///
    /// Uses a variable outflow mass rate to enforce `dP/dt = 0`, maintaining
    /// constant pressure while allowing temperature and density to change.
    ///
    /// # Parameters
    ///
    /// - `volume`: Fixed volume of the control region.
    /// - `state`: Current thermodynamic state of the control volume.
    /// - `inflows`: Incoming flows, each with a mass flow rate and fluid state.
    /// - `heat_input`: Net heat added to the control volume. Positive when heat enters.
    /// - `power_output`: Net work extracted from the control volume. Positive when work leaves.
    ///
    /// # Returns
    ///
    /// The outflow mass rate and the resulting [`StateDerivative<Fluid>`].
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the state derivative cannot be evaluated.
    fn state_derivative_at_constant_pressure(
        &self,
        volume: Constrained<Volume, StrictlyPositive>,
        state: &State<Fluid>,
        inflows: &[Flow<Fluid>],
        heat_input: Power,
        power_output: Power,
    ) -> Result<(Constrained<MassRate, NonNegative>, StateDerivative<Fluid>), PropertyError>;

    /// Computes the time derivative of a control volume with a fixed outflow rate.
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
    /// # Returns
    ///
    /// The resulting [`StateDerivative<Fluid>`] of the control volume.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the state derivative cannot be evaluated.
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
