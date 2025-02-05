use crate::Callable;

/// A trait that extends `Callable` to operate within a context.
///
/// `Context` enables callables to extract input from a structured context,
/// perform a computation, and update the context with the result. This allows
/// state to flow through a chain of callables while keeping transformation
/// logic modular.
///
/// Implementors define:
/// - How to extract input from the context (`extract_input`).
/// - How to apply the computed output back into the context (`apply_output`).
///
/// The `call_with_context` method runs the callable while maintaining context.
///
/// # Example
///
/// ```rust
/// use twine_core::{Callable, Context};
///
/// struct ContextIn {
///     input: i32,
/// }
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct ContextOut {
///     started_from: i32,
///     ended_at: i32,
/// }
///
/// struct AddOne;
///
/// impl Callable for AddOne {
///     type Input = i32;
///     type Output = i32;
///
///     fn call(&self, input: i32) -> i32 {
///         input + 1
///     }
/// }
///
/// impl Context for AddOne {
///     type In = ContextIn;
///     type Out = ContextOut;
///
///     fn extract_input(context: &Self::In) -> Self::Input {
///         context.input
///     }
///
///     fn apply_output(&self, context: Self::In, output: Self::Output) -> Self::Out {
///         Self::Out {
///             started_from: context.input,
///             ended_at: output,
///         }
///     }
/// }
///
/// let ctx_in = ContextIn { input: 10 };
/// let ctx_out = AddOne.call_with_context(ctx_in);
///
/// assert_eq!(
///     ctx_out,
///     ContextOut {
///         started_from: 10,
///         ended_at: 11,
///     }
/// );
/// ```
pub trait Context: Callable {
    type In;
    type Out;

    /// Extracts input for the callable from the given context.
    fn extract_input(context: &Self::In) -> Self::Input;

    /// Applies the callable's output to the context, returning an updated version.
    fn apply_output(&self, context: Self::In, output: Self::Output) -> Self::Out;

    /// Executes the callable within the given context, ensuring data flow is maintained.
    fn call_with_context(&self, context: Self::In) -> Self::Out {
        let input = Self::extract_input(&context);
        let output = self.call(input);
        self.apply_output(context, output)
    }
}
