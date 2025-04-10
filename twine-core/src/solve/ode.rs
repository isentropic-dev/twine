use crate::Component;

/// A trait for components that represent systems of ordinary differential
/// equations (ODEs) with `N` state variables.
///
/// This trait enables a [`Component`] to be integrated using a numerical ODE
/// solver by converting between the solver’s [`State<N>`] representation and
/// the component’s input/output types.
pub trait Integratable<const N: usize>: Component {
    /// Applies solver state to initial conditions to produce the component's input.
    ///
    /// Called at each solver step to update the component input.
    fn apply_state(initial_conditions: &Self::Input, state: State<N>) -> Self::Input;

    /// Extracts solver state from the component's input.
    ///
    /// Called once to determine the initial solver state for integration.
    fn extract_state(input: &Self::Input) -> State<N>;

    /// Extracts state derivatives from the component's output.
    ///
    /// Derivatives must match the order of `y` in [`State`].
    fn extract_derivative(output: &Self::Output) -> [f64; N];
}

/// The state of an ODE system at a single point in the solution.
///
/// This struct holds the independent variable (`x`) and the corresponding
/// values of the dependent state variables (`y`) at that point.
///
/// The order of `y` defines the meaning of each derivative returned by
/// [`Integratable::extract_derivative`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<const N: usize> {
    /// The independent variable (typically time).
    pub x: f64,

    /// The dependent variables at this point in the solution.
    pub y: [f64; N],
}
