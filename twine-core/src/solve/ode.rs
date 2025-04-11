use crate::Component;

/// A trait for components that represent systems of ordinary differential
/// equations (ODEs) with `N` state variables.
///
/// This trait enables integration of a [`Component`] using a numerical ODE
/// solver by mapping between the system’s [`State<N>`], its derivatives, and
/// the component’s input/output types.
pub trait Integratable<const N: usize>: Component {
    /// Constructs the component's input from the system state and initial conditions.
    ///
    /// Called at each solver step to update the component's input.
    fn apply_state(initial_conditions: &Self::Input, state: State<N>) -> Self::Input;

    /// Extracts the system state from the component's input.
    ///
    /// Called once to initialize the state for the solver.
    fn extract_state(input: &Self::Input) -> State<N>;

    /// Extracts the state derivatives from the component’s output.
    ///
    /// Called at each solver step. The order of the returned `dy/dx` values
    /// must match the order of `y` in [`State<N>`].
    fn extract_derivative(output: &Self::Output) -> [f64; N];
}

/// The state of an ODE system at a single point in the solution.
///
/// This struct holds the independent variable (`x`) and the corresponding
/// values of the dependent state variables (`y`) at that point.
///
/// The order of `y` must match the order of the derivative values returned by
/// [`Integratable::extract_derivative`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<const N: usize> {
    /// The independent variable (typically time).
    pub x: f64,

    /// The dependent variables at this point in the solution.
    pub y: [f64; N],
}
