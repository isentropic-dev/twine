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
pub trait StateOperations<Fluid> {}
