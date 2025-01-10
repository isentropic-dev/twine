use super::{Value, ValueKind};

pub(crate) trait Processor: Send {
    /// Computes outputs based on the given inputs.
    ///
    /// # Requirements
    ///
    /// - Must be pure: identical inputs yield identical outputs.
    /// - Internal mutable buffers are allowed but must not affect purity.
    /// - If an error (`Err`) is returned, the `outputs` slice must remain untouched.
    /// - Must write exactly as many values to `outputs` as specified by the
    ///   length of `expected_output_kinds()`. The `outputs` slice is pre-sized
    ///   based on the `expected_output_kinds` definition, and writing an
    ///   incorrect number of values will result in undefined behavior.
    /// - Each value written to `outputs` must match the corresponding kind
    ///   defined by `expected_output_kinds()`.
    /// - The processor must define at least one input kind and one output kind
    ///   through `expected_input_kinds()` and `expected_output_kinds()`.
    ///
    /// # Parameters
    ///
    /// - `inputs`: A slice of input values.
    /// - `outputs`: A mutable slice for storing the computed output values.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Indicates that computation succeeded and `outputs` has been populated.
    /// - `Err(String)`: An error message if computation fails.
    fn compute(&mut self, inputs: &[Value], outputs: &mut [Value]) -> Result<(), String>;

    /// Returns the expected kinds of inputs and their count.
    ///
    /// # Requirements
    ///
    /// - Must return a non-empty slice. A processor must define at least one input kind.
    /// - The `inputs` slice passed to `call` will always have a length equal to
    ///   the number of kinds returned by this method.
    /// - Each value in the `inputs` slice is guaranteed to match its corresponding kind.
    fn expected_input_kinds(&self) -> &[ValueKind];

    /// Returns the expected kinds of outputs and their count.
    ///
    /// # Requirements
    ///
    /// - Must return a non-empty slice. A processor must define at least one output kind.
    /// - The `outputs` slice passed to `call` will always have a length equal
    ///   to the number of kinds returned by this method.
    /// - Implementations of `call` must populate the `outputs` slice with one
    ///   value for each kind returned by this method.
    /// - Each written value must match its corresponding kind.
    fn expected_output_kinds(&self) -> &[ValueKind];
}
