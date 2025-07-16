use twine_core::{
    TimeDifferentiable,
    constraint::{Constrained, NonNegative, StrictlyPositive},
};
use uom::si::f64::{MassRate, Power, Volume};

use crate::{Flow, PropertyError, State, StateDerivative};

/// Thermodynamic operations involving the dynamic evolution of system state.
///
/// The `StateOperations` trait defines how a model applies mass and energy
/// balances to a control volume's [`State<Fluid>`], computing time derivatives
/// based on incoming [`Flow<Fluid>`]s, heat input, and work output.
///
/// This trait provides the foundation for transient or quasi-steady analysis
/// of thermodynamic systems, where the state changes in response to flows and
/// external energy interactions.
///
/// Future methods may include:
/// - Computing [`StateDerivative<Fluid>`] from inflows, heat, and work
/// - Capturing fluid-specific dynamics like composition changes or mixing effects
///
/// See [`FlowOperations`] for steady-flow modeling across control boundaries.
pub trait StateOperations<Fluid: TimeDifferentiable> {
    /// Computes the state derivative of a control volume at constant density.
    ///
    /// # Assumptions
    ///
    /// - The control volume is fixed in size and fully mixed.
    /// - Outflow exits at the current state of the control volume.
    /// - Density is maintained by adjusting the outflow rate.
    ///
    /// # Responsibilities of the Implementor
    ///
    /// - Apply conservation of mass and energy across the control volume.
    /// - Compute the outflow rate such that the time derivative of density is zero.
    /// - If the model supports pressure, it may optionally verify that each inflow
    ///   has pressure ≥ the state pressure to ensure physically valid mixing.
    ///
    /// # Sign Conventions
    ///
    /// - `heat_input` is positive when heat enters the control volume,
    ///   negative when it leaves.
    /// - `power_output` is positive when work is extracted from the control volume,
    ///   negative when work is done on it.
    ///
    /// # Parameters
    ///
    /// - `volume`: Size of the control volume.
    /// - `state`: Current thermodynamic state of the control volume.
    /// - `inflows`: Incoming flows, each with a mass flow and associated state.
    /// - `heat_input`: Net heat transfer into the control volume.
    /// - `power_output`: Net work extracted from the control volume.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The computed outflow mass rate.
    /// - The time derivative of the control volume state.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if inputs are invalid or required properties
    /// cannot be computed.
    fn state_derivative_at_constant_density(
        &self,
        volume: Constrained<Volume, StrictlyPositive>,
        state: &State<Fluid>,
        inflows: &[Flow<Fluid>],
        heat_input: Power,
        power_output: Power,
    ) -> Result<(Constrained<MassRate, NonNegative>, StateDerivative<Fluid>), PropertyError>;
}
