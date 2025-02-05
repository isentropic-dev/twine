use crate::Callable;

/// A trait that enables chaining two `Callable` types together.
///
/// The `Then` trait allows one `Callable` to be tied to another, ensuring that
/// the output of the first callable can be used as the input for the next.
/// This allows callables to be woven together seamlessly, forming a functional
/// sequence.
///
/// Instead of requiring an exact type match (`A::Output == B::Input`), this
/// trait leverages Rust’s `From` trait to allow seamless conversion between
/// output and input types. If `B::Input` implements `From<A::Output>`, the
/// conversion happens automatically, enabling flexible chaining of callables.
///
/// When working with standard library types, you may need to define a wrapper
/// type instead of implementing `From` directly due to Rust’s orphan rule.
///
/// # Blanket Implementation
///
/// You do not need to implement `Then` manually. Any `Callable` type with a
/// convertible output automatically supports `.then()`, eliminating boilerplate
/// and enabling seamless composition.
///
/// # Example
///
/// ```rust
/// use twine_core::{Callable, Then};
///
/// // Why do we need this?
/// // Rust’s orphan rule prevents implementing `From<String>` for `i32`
/// // directly, so we use a newtype wrapper.
/// struct MyInteger(i32);
///
/// impl From<String> for MyInteger {
///     fn from(value: String) -> Self {
///         MyInteger(value.parse::<i32>().unwrap_or(0))
///     }
/// }
///
/// struct ToStringDoubled;
///
/// impl Callable for ToStringDoubled {
///     type Input = i32;
///     type Output = String;
///
///     fn call(&self, input: i32) -> String {
///         (input * 2).to_string()
///     }
/// }
///
/// struct ParseToInteger;
///
/// impl Callable for ParseToInteger {
///     type Input = MyInteger;
///     type Output = i32;
///
///     fn call(&self, input: MyInteger) -> i32 {
///         input.0
///     }
/// }
///
/// let chain = ToStringDoubled.then(ParseToInteger);
/// let result = chain.call(21);
/// assert_eq!(result, 42);
/// ```
pub trait Then<C>
where
    Self: Callable,
    C: Callable<Input: From<Self::Output>>,
{
    type Then: Callable<Input = Self::Input, Output = C::Output>;

    /// Ties the current `Callable` to another, producing a new composed callable.
    ///
    /// The returned `Self::Then` ensures that the overall sequence maintains
    /// a consistent input-output flow, automatically converting `A::Output`
    /// to `B::Input` when possible.
    fn then(self, callable: C) -> Self::Then;
}

/// A `Callable` that represents the sequential execution of two callables.
///
/// `Twine<A, B>` ties two `Callable` instances together, passing the output
/// of `A` as the input to `B`. It is automatically created when `.then()` is
/// called, making composition intuitive.
pub struct Twine<A, B> {
    first: A,
    second: B,
}

impl<A, B> Callable for Twine<A, B>
where
    A: Callable,
    B: Callable<Input: From<A::Output>>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn call(&self, input: Self::Input) -> Self::Output {
        let first_output = self.first.call(input);
        let second_input = first_output.into();
        self.second.call(second_input)
    }
}

/// Blanket implementation of `Then` for any compatible `Callable` instances.
///
/// This implementation allows any `Callable` to be tied to another using
/// `.then()`, as long as their output and input types are compatible.
impl<A, B> Then<B> for A
where
    A: Callable,
    B: Callable<Input: From<A::Output>>,
{
    type Then = Twine<A, B>;

    fn then(self, callable: B) -> Self::Then {
        Twine {
            first: self,
            second: callable,
        }
    }
}
