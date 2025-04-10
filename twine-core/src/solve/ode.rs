use crate::Component;

/// A trait for components that represent systems of ordinary differential
/// equations (ODEs) with `N` state variables.
///
/// This trait enables a [`Component`] to be integrated using a numerical ODE
/// solver by converting between the solver’s [`State<N>`] representation and
/// the component’s input/output types.
pub trait Integratable<const N: usize>: Component {
    /// Constructs the component's input by applying the given solver state to
    /// the provided initial conditions.
    ///
    /// Called at each solver step to update the component input.
    fn apply_state(initial_conditions: &Self::Input, state: State<N>) -> Self::Input;

    /// Extracts the solver state from the component's input.
    ///
    /// Called once at the start of integration to determine initial state values.
    fn extract_state(input: &Self::Input) -> State<N>;

    /// Extracts the state derivatives from the component's output.
    ///
    /// The returned array must align with the order of `y` in [`State`].
    fn extract_derivative(output: &Self::Output) -> [f64; N];
}

/// The state of an ODE system at a given point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<const N: usize> {
    /// The independent variable (typically time).
    pub x: f64,

    /// The dependent variables at this point.
    ///
    /// The order of values must match the derivative array returned by
    /// [`Integratable::extract_derivative`].
    pub y: [f64; N],
}
