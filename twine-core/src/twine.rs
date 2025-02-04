use crate::Callable;

/// A trait that enables tying together two `Callable` components.
///
/// The `Then` trait allows one `Callable` to be tied to another, ensuring that
/// the output of the first component can be used as the input for the next.
///
/// Instead of requiring an exact type match (`A::Output == B::Input`), this
/// trait leverages Rust’s `From` trait to allow seamless conversion between
/// output and input types. If `B::Input` implements `From<A::Output>`, the
/// conversion happens automatically, enabling flexible chaining of components.
///
/// When working with standard library types, you may need to define a wrapper
/// type instead of implementing `From` directly due to Rust’s orphan rule.
///
/// # Blanket Implementation
///
/// You do not need to implement `Then` manually. Any `Callable` type that
/// produces an output convertible to another’s input will automatically support
/// `.then()`. This keeps composition seamless and eliminates boilerplate code.
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
///
/// ```
pub trait Then<C>
where
    Self: Callable,
    C: Callable<Input: From<Self::Output>>,
{
    type Then: Callable<Input = Self::Input, Output = C::Output>;

    /// Ties the current `Callable` to another, producing a new composed component.
    ///
    /// The returned `Self::Then` ensures that the overall sequence maintains
    /// a consistent input-output flow, automatically converting `A::Output` to
    /// `B::Input` when possible.
    fn then(self, component: C) -> Self::Then;
}

/// A `Callable` that represents the sequential execution of two components.
///
/// `Twine<A, B>` ties two `Callable` components together, passing the output
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

/// Blanket implementation of `Then` for any compatible `Callable` components.
///
/// This implementation allows any `Callable` component to be tied together with
/// another using `.then()`, as long as their output and input types are compatible.
impl<A, B> Then<B> for A
where
    A: Callable,
    B: Callable<Input: From<A::Output>>,
{
    type Then = Twine<A, B>;

    fn then(self, component: B) -> Self::Then {
        Twine {
            first: self,
            second: component,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AddOne;

    impl Callable for AddOne {
        type Input = i32;
        type Output = i32;

        fn call(&self, input: i32) -> i32 {
            input + 1
        }
    }

    struct MultiplyBy {
        factor: i32,
    }

    impl Callable for MultiplyBy {
        type Input = i32;
        type Output = i32;

        fn call(&self, input: i32) -> i32 {
            input * self.factor
        }
    }

    struct ToFloat;

    impl Callable for ToFloat {
        type Input = i32;
        type Output = f64;

        fn call(&self, input: i32) -> f64 {
            f64::from(input)
        }
    }

    struct IncreaseBySmallAmount;

    impl Callable for IncreaseBySmallAmount {
        type Input = f64;
        type Output = f64;

        fn call(&self, input: f64) -> f64 {
            input + 0.1
        }
    }

    struct RoundToInt;

    impl Callable for RoundToInt {
        type Input = f64;
        type Output = i32;

        #[allow(clippy::cast_possible_truncation)]
        fn call(&self, input: f64) -> i32 {
            input.round() as i32
        }
    }

    struct IsPositive;

    impl Callable for IsPositive {
        type Input = i32;
        type Output = bool;

        fn call(&self, input: i32) -> bool {
            input > 0
        }
    }

    #[test]
    fn twine_execution() {
        let twine = AddOne
            .then(MultiplyBy { factor: 5 })
            .then(AddOne)
            .then(AddOne);
        assert_eq!(twine.call(7), 42); // (7 + 1) * 5 + 1 + 1 = 42
    }

    #[test]
    fn type_transformation() {
        let twine = AddOne
            .then(ToFloat)
            .then(IncreaseBySmallAmount)
            .then(RoundToInt);
        assert_eq!(twine.call(3), 4); // 3 + 1 -> 4.0 + 0.1 -> 4
    }

    #[test]
    fn boolean_output() {
        let twine = AddOne.then(AddOne).then(IsPositive);
        assert!(twine.call(-1)); //  -1 + 1 + 1 =  1 (true)
        assert!(!twine.call(-3)); // -3 + 1 + 1 = -1 (false)
    }

    #[test]
    fn two_chains_tied_together() {
        let add_four = AddOne.then(AddOne).then(AddOne).then(AddOne);
        let double_it = MultiplyBy { factor: 2 };

        let chain_one = add_four.then(MultiplyBy { factor: 3 });
        let chain_two = AddOne.then(double_it);

        let combined = chain_one.then(chain_two);

        assert_eq!(combined.call(0), 26); // (((0 + 4) * 3) + 1) * 2 = 26
    }
}
