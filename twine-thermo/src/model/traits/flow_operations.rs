/// Thermodynamic operations involving steady-state fluid flows.
///
/// The `FlowOperations` trait defines how a model applies mass and energy
/// balances to one or more [`Flow<Fluid>`]s, with optional heat and work terms.
///
/// This trait provides the foundation for modeling open-system thermodynamics,
/// including mixing, heating, and flow-based energy exchanges.
///
/// Future methods may include:
/// - Combining inflows with heat and work to compute an output flow
/// - Solving for required heat input given an outflow
/// - Accounting for mixing behavior or reaction enthalpy in stateful fluids
///
/// See [`StateOperations`] for control volume dynamics involving `State<Fluid>`.
pub trait FlowOperations<Fluid> {}
