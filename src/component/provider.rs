use super::{value, CallFn, Component, Value, ValueKind};

/// A trait for creating `Component` instances.
///
/// Implementors of the `Provider` trait define the expected types of inputs
/// and outputs (`ValueKind`) and supply the computation logic (`CallFn`) that
/// transforms inputs into outputs.
///
/// # Responsibilities
///
/// - Specify the kinds of inputs required for the component.
/// - Specify the kinds of outputs the component will produce.
/// - Provide a pure function that performs the computation based on the inputs.
pub(crate) trait Provider {
    /// Returns the function responsible for computing outputs from inputs.
    ///
    /// The returned `CallFn` must adhere to the following requirements:
    ///
    /// # Requirements
    ///
    /// - It must be invoked with exactly the number and kinds of inputs
    ///   specified by `expected_inputs()`.
    /// - Each input must match the corresponding kind in `expected_inputs()`.
    /// - Upon successful execution, the function must overwrite all entries
    ///   in the `outputs` slice, ensuring they match the number and kinds
    ///   specified by `expected_outputs()`.
    /// - If an error occurs during execution, `outputs` must remain unchanged.
    /// - The function must be pure, producing outputs solely based on the
    ///   provided inputs. Identical inputs must always result in identical
    ///   outputs, regardless of any external state.
    ///
    /// Implementations may utilize internal mutable buffers, caching
    /// mechanisms, or external resources to perform computations, provided
    /// these do not compromise the function's purity.
    fn provide_call_fn(&self) -> CallFn;

    /// Returns a slice representing the expected input kinds for the component.
    ///
    /// The order of `ValueKind` elements corresponds to the expected order
    /// of input values. The length of the returned slice determines the exact
    /// number of inputs required.
    fn expected_inputs(&self) -> &[ValueKind];

    /// Returns a slice representing the expected outputs produced by the component.
    ///
    /// The order of `ValueKind` elements corresponds to the expected order of
    /// output values. The length of the returned slice determines the exact
    /// number of outputs the component will produce.
    fn expected_outputs(&self) -> &[ValueKind];

    /// Creates a new `Component` with initial inputs.
    ///
    /// This method performs several steps:
    ///
    /// - Validates that `inputs` match the expected kinds from `expected_inputs()`.
    /// - Allocates a mutable slice for outputs based on `expected_outputs()`.
    /// - Invokes the `CallFn` to compute outputs from the validated inputs.
    /// - Verifies that the computed outputs match the expected kinds.
    ///
    /// # Returns
    ///
    /// - `Ok(Component)` if all inputs are valid and the `CallFn` successfully
    ///   computes outputs that match the expected kinds.
    /// - `Err(Error)` if input validation fails, the initial computation fails,
    ///   or output validation fails.
    fn create_component(&self, inputs: Vec<Value>) -> Result<Component, Error> {
        value::validate_kinds(
            inputs.iter().map(ValueKind::from),
            self.expected_inputs().iter().copied(),
        )
        .map_err(Error::IncorrectInputs)?;

        let call_fn = self.provide_call_fn();
        let mut outputs = vec![Value::Number(0.0); self.expected_outputs().len()];
        call_fn(&inputs, &mut outputs).map_err(Error::InitialCallFailed)?;

        value::validate_kinds(
            outputs.iter().map(ValueKind::from),
            self.expected_outputs().iter().copied(),
        )
        .map_err(Error::UnexpectedOutputs)?;

        Ok(Component {
            call_fn,
            inputs: inputs.into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
        })
    }
}

/// Represents errors that can occur during the creation of a `Component`.
#[derive(Debug)]
pub(crate) enum Error {
    /// The provided input values do not match the expected kinds.
    IncorrectInputs(value::SliceKindError),
    /// The initial outputs produced do not match the expected kinds.
    UnexpectedOutputs(value::SliceKindError),
    /// The initial call to compute outputs failed.
    InitialCallFailed(String),
}
