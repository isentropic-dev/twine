use crate::{DerivativeOf, StepIntegrable};

/// Defines an ODE (ordinary differential equation) problem to be solved.
///
/// An ODE problem extracts a state from model input, computes derivatives from
/// model input and output, and reconstructs model input from an updated state
/// and step size. This trait enables generic ODE solvers to integrate any model
/// by working with a state type that implements [`StepIntegrable`].
pub trait OdeProblem {
    type Input;
    type Output;
    type Delta;
    type State: StepIntegrable<Self::Delta>;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Extracts the state from model input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the state cannot be extracted from the input.
    fn state(&self, input: &Self::Input) -> Result<Self::State, Self::Error>;

    /// Computes the derivative of the state from model input and output.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the derivative cannot be computed.
    fn derivative(
        &self,
        input: &Self::Input,
        output: &Self::Output,
    ) -> Result<DerivativeOf<Self::State, Self::Delta>, Self::Error>;

    /// Builds model input from a state and step size.
    ///
    /// This reconstructs the full model input after the state has been stepped.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the input cannot be constructed from the state.
    fn build_input(
        &self,
        base: &Self::Input,
        state: &Self::State,
        delta: &Self::Delta,
    ) -> Result<Self::Input, Self::Error>;

    /// Finalizes input after a successful integration step.
    ///
    /// This is called only after a step is accepted by the solver. It provides
    /// a hook for constraint enforcement, accumulated error correction, or other
    /// problem-specific adjustments. For example, this is where discrete controls
    /// (valve openings, mode switches) could be applied in transient simulations.
    ///
    /// The default implementation returns the input unchanged. Only implement
    /// this method if your problem requires it.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if finalization fails.
    fn finalize_step(
        &self,
        next_input: Self::Input,
        _prev_input: &Self::Input,
        _prev_output: &Self::Output,
        _step_delta: &Self::Delta,
    ) -> Result<Self::Input, Self::Error> {
        Ok(next_input)
    }
}
