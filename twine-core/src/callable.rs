/// A trait representing a function-like component in Twine.
///
/// A `Callable` component is the core building block for composition in Twine.
/// Implementations must be deterministic, meaning the same input must always
/// produce the same output.
///
/// Any `Callable` can be tied to another `Callable` using `.then()`, as long
/// as the output of the first component is compatible with the input of the
/// next. If the input type implements `From` for the previous output type, the
/// conversion happens automatically.
///
/// The `.then()` method is provided by the `Then` trait, which is implemented
/// automatically for all compatible `Callable` types. You do not need to
/// implement `Then` manually—any `Callable` will support `.then()` as long as
/// its output can be converted into the next component’s input.
///
/// # Example
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
