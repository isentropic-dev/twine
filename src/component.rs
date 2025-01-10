mod buffer;
mod metadata;
mod processor;

use processor::Processor;

pub(crate) struct Component<P: Processor> {
    pub(crate) inputs: Box<[Value]>,
    pub(crate) outputs: Box<[Value]>,
    processor: P,
}

impl<P: Processor> Component<P> {
    /// Creates a new `Component` instance.
    ///
    /// Initializes the component's inputs and outputs based on the processor's
    /// expected input and output kinds. Performs an initial update using the
    /// processor to populate the outputs.
    ///
    /// In debug builds, verifies that the processor defines at least one
    /// expected input and output kind and validates that the provided inputs
    /// match the processor's expected input kinds.
    ///
    /// # Parameters
    ///
    /// - `inputs`: A slice containing the initial input values.
    /// - `processor`: Defines input/output kinds and performs the computation.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: A new `Component` instance.
    /// - `Err(String)`: Returned if the processor's initial computation fails.
    ///
    /// # Debug Assertions
    ///
    /// - In debug builds, panics if:
    ///   - The processor does not define at least one input and one output kind.
    ///   - The provided `inputs` do not match the processor's expected input kinds.
    ///   - The computed `outputs` do not match the processor's expected output kinds.
    ///
    /// # Undefined Behavior
    ///
    /// - In release builds, it is the responsibility of the system using the
    ///   `Component` to ensure that:
    ///   - The processor defines at least one input and output kind.
    ///   - The `inputs` match the processor's expected input kinds.
    ///
    /// Failure to adhere to these requirements results in undefined behavior.
    pub(crate) fn new(inputs: &[Value], processor: P) -> Result<Self, String> {
        debug_assert!(
            !processor.expected_input_kinds().is_empty(),
            "Processor must have at least one expected input kind."
        );
        debug_assert!(
            !processor.expected_output_kinds().is_empty(),
            "Processor must have at least one expected output kind."
        );

        let mut component = Self {
            inputs: default_values_from_kinds(processor.expected_input_kinds()),
            outputs: default_values_from_kinds(processor.expected_output_kinds()),
            processor,
        };

        component.update(inputs)?;

        Ok(component)
    }

    /// Updates the component with new input values.
    ///
    /// Computes new outputs using the processor and updates the component's
    /// state atomically. In debug builds, validates that the inputs match the
    /// processor's expected input kinds and that the computed outputs match the
    /// expected output kinds. If an error occurs, the component's state (inputs
    /// and outputs) remains unchanged.
    ///
    /// # Parameters
    ///
    /// - `inputs`: A slice of new input values.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the update completes successfully.
    /// - `Err(String)`: If the processor's computation fails during the update.
    ///
    /// # Debug Assertions
    ///
    /// - In debug builds, panics if:
    ///   - The provided `inputs` do not match the processor's expected input kinds.
    ///   - The computed `outputs` do not match the processor's expected output kinds.
    ///
    /// # Undefined Behavior
    ///
    /// - In release builds, it is the responsibility of the system using the
    ///   `Component` to ensure that:
    ///   - The `inputs` match the processor's expected input kinds.
    ///   - The `outputs` are consumed in a manner consistent with the
    ///     processor's expected output kinds.
    ///
    /// Failure to adhere to these requirements results in undefined behavior.
    pub(crate) fn update(&mut self, inputs: &[Value]) -> Result<(), String> {
        if cfg!(debug_assertions) {
            assert_value_kinds(inputs, self.processor.expected_input_kinds(), "Input");
        }

        self.processor.compute(inputs, &mut self.outputs)?;
        self.inputs.copy_from_slice(inputs);

        if cfg!(debug_assertions) {
            assert_value_kinds(
                &self.outputs,
                self.processor.expected_output_kinds(),
                "Output",
            );
        }

        Ok(())
    }
}

/// A value used as input or output by a `Component`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Value {
    /// A boolean value (`true` or `false`).
    Boolean(bool),
    /// A 32-bit signed integer.
    Integer(i32),
    /// A 64-bit floating-point number.
    Number(f64),
}

/// Describes the type of a `Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    /// A boolean.
    Boolean,
    /// A 32-bit signed integer.
    Integer,
    /// A 64-bit floating-point number.
    Number,
}

impl From<&Value> for ValueKind {
    fn from(value: &Value) -> Self {
        match value {
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Integer(_) => ValueKind::Integer,
            Value::Number(_) => ValueKind::Number,
        }
    }
}

/// Creates default `Value` instances for each `ValueKind` in the provided slice.
fn default_values_from_kinds(kinds: &[ValueKind]) -> Box<[Value]> {
    kinds
        .iter()
        .map(|kind| match kind {
            ValueKind::Boolean => Value::Boolean(false),
            ValueKind::Integer => Value::Integer(0),
            ValueKind::Number => Value::Number(0.0),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

/// Validates that a slice of `Value` matches the expected kinds.
///
/// # Parameters
///
/// - `values`: A slice of `Value` to validate.
/// - `expected_kinds`: A slice of `ValueKind` representing the expected kinds.
/// - `context`: A string describing the validation context (e.g., "Input" or "Output").
///
/// # Panics
///
/// Panics if the number of values does not match the number of expected kinds,
/// or if the kinds of `values` do not match `expected_kinds`.
fn assert_value_kinds(values: &[Value], expected_kinds: &[ValueKind], context: &str) {
    let kinds: Vec<_> = values.iter().map(ValueKind::from).collect();
    assert_eq!(
        kinds, expected_kinds,
        "{context} validation failed: expected {expected_kinds:?}, but received {kinds:?}.",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock implementation of the `Processor` trait for testing.
    ///
    /// - Expects two inputs: an integer and a boolean.
    /// - Produces one output: a floating-point number.
    /// - Behavior:
    ///   - If the integer input is `26` and the boolean input is `true`,
    ///     it returns an error: `"Invalid input combination: 26 and true"`.
    ///   - Otherwise, it computes the output as:
    ///     - `inputs[0]` cast to `f64`, doubled if `inputs[1]` is `true`.
    struct MockProcessor;

    impl Processor for MockProcessor {
        fn expected_input_kinds(&self) -> &[ValueKind] {
            &[ValueKind::Integer, ValueKind::Boolean]
        }

        fn expected_output_kinds(&self) -> &[ValueKind] {
            &[ValueKind::Number]
        }

        fn compute(&mut self, inputs: &[Value], outputs: &mut [Value]) -> Result<(), String> {
            let i = match &inputs[0] {
                Value::Integer(val) => *val,
                _ => unreachable!("Expected an integer as the first input"),
            };
            let b = match &inputs[1] {
                Value::Boolean(val) => *val,
                _ => unreachable!("Expected a boolean as the second input"),
            };

            if i == 26 && b {
                return Err("Invalid input combination: 26 and true".into());
            }

            outputs[0] = Value::Number(if b { (i * 2).into() } else { (i).into() });

            Ok(())
        }
    }

    fn create_mock_component(inputs: &[Value]) -> Result<Component<MockProcessor>, String> {
        Component::new(inputs, MockProcessor)
    }

    #[test]
    fn valid_initialization() -> Result<(), String> {
        let component = create_mock_component(&[Value::Integer(5), Value::Boolean(true)])?;
        assert_eq!(component.outputs[0], Value::Number(10.0));

        let component = create_mock_component(&[Value::Integer(2), Value::Boolean(false)])?;
        assert_eq!(component.outputs[0], Value::Number(2.0));

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Input validation failed")]
    fn invalid_initialization_due_to_length_mismatch() {
        create_mock_component(&[Value::Integer(0)]).unwrap();
    }

    #[test]
    #[should_panic(expected = "Input validation failed")]
    fn invalid_initialization_due_to_kind_mismatch() {
        create_mock_component(&[Value::Boolean(true), Value::Integer(0)]).unwrap();
    }

    #[test]
    fn multiple_updates_with_valid_inputs() -> Result<(), String> {
        let mut component = create_mock_component(&[Value::Integer(1), Value::Boolean(true)])?;
        assert_eq!(component.outputs[0], Value::Number(2.0));

        component.update(&[Value::Integer(15), Value::Boolean(false)])?;
        assert_eq!(component.outputs[0], Value::Number(15.0));

        component.update(&[Value::Integer(10), Value::Boolean(false)])?;
        assert_eq!(component.outputs[0], Value::Number(10.0));

        component.update(&[Value::Integer(7), Value::Boolean(true)])?;
        assert_eq!(component.outputs[0], Value::Number(14.0));

        component.update(&[Value::Integer(100), Value::Boolean(false)])?;
        assert_eq!(component.outputs[0], Value::Number(100.0));

        Ok(())
    }

    #[test]
    fn inputs_and_outputs_remain_unchanged_on_error() -> Result<(), String> {
        let mut component = create_mock_component(&[Value::Integer(10), Value::Boolean(false)])?;

        let original_inputs = component.inputs.clone();
        let original_outputs = component.outputs.clone();

        let result = component.update(&[Value::Integer(26), Value::Boolean(true)]);
        assert!(result.is_err());
        assert_eq!(component.inputs, original_inputs);
        assert_eq!(component.outputs, original_outputs);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "Input validation failed")]
    fn update_with_invalid_input_kinds() {
        create_mock_component(&[Value::Integer(10), Value::Boolean(false)])
            .unwrap()
            .update(&[Value::Boolean(true), Value::Integer(10)])
            .unwrap();
    }
}
