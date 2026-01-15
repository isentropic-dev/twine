/// A callable model that maps an input to an output.
pub trait Model {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Calls the model with the given input.
    ///
    /// # Errors
    ///
    /// Returns an error if the call fails.
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
