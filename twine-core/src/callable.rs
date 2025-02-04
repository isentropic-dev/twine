/// A trait representing a function-like component in Twine.
///
/// A `Callable` component is the core building block for composition in Twine.
/// Implementations must be deterministic, meaning the same input must always
/// produce the same output.
///
/// Any `Callable` can be tied to another `Callable` using `.then()`, as long
/// as the output of the first component can be used as the input for the next.
/// This functionality is provided automatically via a blanket implementation of
/// the `Then` trait.
///
/// ```rust
/// use twine_core::{Callable, Then};
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
/// struct MultiplyBy {
///     factor: i32,
/// }
///
/// impl Callable for MultiplyBy {
///     type Input = i32;
///     type Output = i32;
///
///     fn call(&self, input: i32) -> i32 {
///         input * self.factor
///     }
/// }
///
/// let double_it = MultiplyBy { factor: 2 };
/// let triple_it = MultiplyBy { factor: 3 };
///
/// let chain = double_it.then(AddOne).then(triple_it).then(AddOne);
/// let result = chain.call(2);
/// assert_eq!(result, 16);
/// ```
pub trait Callable {
    type Input;
    type Output;

    fn call(&self, input: Self::Input) -> Self::Output;
}
