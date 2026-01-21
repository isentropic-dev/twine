/// A callable model that maps a typed input to a typed output.
///
/// Models must be deterministic, always producing the same result for a given
/// input, which makes them a stable foundation for solvers, simulations,
/// caching, and instrumentation.
pub trait Model {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Calls the model with the given input.
    ///
    /// # Errors
    ///
    /// Each model defines its own `Error` type to represent domain-specific failures.
    fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
}

/// A captured input/output pair from a model call.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot<I, O> {
    pub input: I,
    pub output: O,
}

impl<I, O> Snapshot<I, O> {
    /// Creates a new snapshot from input and output values.
    pub fn new(input: I, output: O) -> Self {
        Self { input, output }
    }
}
