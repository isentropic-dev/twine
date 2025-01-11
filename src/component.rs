mod buffer;
mod metadata;
mod provider;
mod value;

pub(crate) use value::{Value, ValueKind};

/// A boxed function that computes outputs from inputs.
///
/// # Requirements
///
/// - If an error (`Err`) is returned, the `outputs` slice remains unchanged.
/// - On success (`Ok(())`), the function fully overwrites the `outputs` slice.
pub(crate) type CallFn = Box<dyn Fn(&[Value], &mut [Value]) -> Result<(), String>>;

/// Represents a component with inputs, outputs, and a computation function.
///
/// Maintains the current inputs and outputs, and uses `CallFn` to compute new
/// outputs based on the inputs.
pub(crate) struct Component {
    call_fn: CallFn,
    inputs: Box<[Value]>,
    outputs: Box<[Value]>,
}

impl Component {
    /// Updates the component with new input values and computes outputs.
    ///
    /// If an error occurs during computation, the component’s inputs and
    /// outputs remain unchanged; otherwise, both are updated atomically.
    ///
    /// # Parameters
    ///
    /// - `inputs`: A slice of new input values.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the computation is successful.
    /// - `Err(String)` if the computation fails.
    ///
    /// # Panics (Debug Build Only)
    ///
    /// - If `inputs` do not match the expected kinds.
    /// - If computed outputs do not match the expected kinds.
    ///
    /// # Behavior (Release Build)
    ///
    /// Caller must ensure `inputs` match the expected kinds; otherwise,
    /// behavior is unspecified.
    pub(crate) fn call(&mut self, inputs: &[Value]) -> Result<(), String> {
        if cfg!(debug_assertions) {
            value::validate_kinds(
                inputs.iter().map(ValueKind::from),
                self.inputs.iter().map(ValueKind::from),
            )
            .expect("Input kinds do not match the expected kinds.");

            let mut outputs = self.outputs.clone();
            (self.call_fn)(inputs, &mut outputs)?;

            value::validate_kinds(
                outputs.iter().map(ValueKind::from),
                self.outputs.iter().map(ValueKind::from),
            )
            .expect("Output kinds do not match the expected kinds.");

            self.outputs.copy_from_slice(&outputs);
        } else {
            (self.call_fn)(inputs, &mut self.outputs)?;
        }

        self.inputs.copy_from_slice(inputs);
        Ok(())
    }

    /// Returns a copy of the input at the given index, or `None` if out of bounds.
    pub(crate) fn get_input(&self, index: usize) -> Option<Value> {
        self.inputs.get(index).copied()
    }

    /// Returns a copy of the input at the given index without bounds checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure the input index is within valid range; otherwise this is
    /// undefined behavior.
    pub(crate) unsafe fn get_input_unchecked(&self, index: usize) -> Value {
        *self.inputs.get_unchecked(index)
    }

    /// Returns a copy of the output at the given index, or `None` if out of bounds.
    pub(crate) fn get_output(&self, index: usize) -> Option<Value> {
        self.outputs.get(index).copied()
    }

    /// Returns a copy of the output at the given index without bounds checks.
    ///
    /// # Safety
    ///
    /// Caller must ensure the output index is within valid range; otherwise this is
    /// undefined behavior.
    pub(crate) unsafe fn get_output_unchecked(&self, index: usize) -> Value {
        *self.outputs.get_unchecked(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use provider::{Error as ProviderError, Provider};

    /// A mock component provider for testing purposes.
    ///
    /// - Expects two inputs: an integer and a boolean.
    /// - Produces one output: a floating-point number.
    /// - Behavior:
    ///   - If the integer input is `26` and the boolean input is `true`,
    ///     it returns an error: `"Invalid input combination: 26 and true"`.
    ///   - Otherwise, it computes the output as:
    ///     - `inputs[0]` cast to `f64`, doubled if `inputs[1]` is `true`.
    struct MockProvider;

    impl Provider for MockProvider {
        fn expected_inputs(&self) -> &[ValueKind] {
            &[ValueKind::Integer, ValueKind::Boolean]
        }

        fn expected_outputs(&self) -> &[ValueKind] {
            &[ValueKind::Number]
        }

        fn provide_call_fn(&self) -> CallFn {
            Box::new(|inputs, outputs| {
                let i = match inputs.first() {
                    Some(Value::Integer(val)) => *val,
                    _ => return Err("Expected an integer as the first input.".to_string()),
                };
                let b = match inputs.get(1) {
                    Some(Value::Boolean(val)) => *val,
                    _ => return Err("Expected a boolean as the second input.".to_string()),
                };

                if i == 26 && b {
                    return Err("Invalid input combination: 26 and true".to_string());
                }

                outputs[0] = Value::Number(if b { (i * 2).into() } else { i.into() });
                Ok(())
            })
        }
    }

    #[test]
    fn create_component() {
        let component = MockProvider
            .create_component(vec![5.into(), true.into()])
            .unwrap();

        assert_eq!(component.inputs.len(), 2);
        assert_eq!(component.outputs.len(), 1);

        assert_eq!(component.get_input(0), Some(5.into()));
        assert_eq!(component.get_input(1), Some(true.into()));
        assert_eq!(component.get_output(0), Some(10.0.into()));

        assert_eq!(unsafe { component.get_input_unchecked(0) }, 5.into());
        assert_eq!(unsafe { component.get_input_unchecked(1) }, true.into());
        assert_eq!(unsafe { component.get_output_unchecked(0) }, 10.0.into());
    }

    #[test]
    fn create_component_with_invalid_initial_inputs() {
        assert!(matches!(
            MockProvider.create_component(vec![5.into()]),
            Err(ProviderError::IncorrectInputs(_))
        ));

        assert!(matches!(
            MockProvider.create_component(vec![true.into()]),
            Err(ProviderError::IncorrectInputs(_))
        ));

        assert!(matches!(
            MockProvider.create_component(vec![true.into(), 5.into()]),
            Err(ProviderError::IncorrectInputs(_))
        ));
    }

    #[test]
    fn create_component_with_first_call_failure() {
        let result = MockProvider.create_component(vec![26.into(), true.into()]);

        assert!(result.is_err());
        if let Err(provider::Error::InitialCallFailed(message)) = result {
            assert_eq!(message, "Invalid input combination: 26 and true");
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn multiple_updates_with_valid_inputs() {
        let mut component = MockProvider
            .create_component(vec![10.into(), false.into()])
            .unwrap();
        assert_eq!(component.get_input(0), Some(10.into()));
        assert_eq!(component.get_input(1), Some(false.into()));
        assert_eq!(component.get_output(0), Some(10.0.into()));

        component.call(&[7.into(), true.into()]).unwrap();
        assert_eq!(unsafe { component.get_input_unchecked(0) }, 7.into());
        assert_eq!(unsafe { component.get_input_unchecked(1) }, true.into());
        assert_eq!(unsafe { component.get_output_unchecked(0) }, 14.0.into());

        component.call(&[3.into(), false.into()]).unwrap();
        assert_eq!(component.get_output(0), Some(3.0.into()));
    }

    #[test]
    fn inputs_and_outputs_remain_unchanged_on_error() {
        let mut component = MockProvider
            .create_component(vec![1.into(), false.into()])
            .unwrap();

        let original_inputs = component.inputs.clone();
        let original_outputs = component.outputs.clone();

        let result = component.call(&[26.into(), true.into()]);
        assert!(result.is_err());

        assert_eq!(component.inputs.as_ref(), original_inputs.as_ref());
        assert_eq!(component.outputs.as_ref(), original_outputs.as_ref());
    }

    #[test]
    #[should_panic(expected = "Input kinds do not match the expected kinds.")]
    fn call_with_invalid_input_length() {
        let mut component = MockProvider
            .create_component(vec![1.into(), false.into()])
            .unwrap();

        component.call(&[5.into()]).unwrap();
    }
}
