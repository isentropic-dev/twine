use crate::{Callable, Context};

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

/// A `Callable` that sequentially calls two `Callable`s.
///
/// `Twine<A, B>` passes `A`'s output to `B`'s input and is created
/// automatically via `.then()`, making function composition seamless.
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

/// Implements `Context` for a `Twine<A, B>`, allowing two `Context`-aware
/// callables to process structured contexts in sequence.
///
/// The first callable (`A`) extracts input, computes a result, and updates the
/// context. The updated context is passed to the second callable (`B`), which
/// produces the final transformed context.
impl<A, B> Context for Twine<A, B>
where
    A: Context + Callable,
    B: Context + Callable,
    A::Out: Into<B::In>,
    <B as Callable>::Input: From<<A as Callable>::Output>,
{
    type In = A::In;
    type Out = B::Out;

    fn extract_input(context: &Self::In) -> Self::Input {
        A::extract_input(context)
    }

    fn apply_output(&self, context: Self::In, output: Self::Output) -> Self::Out {
        let context = self.first.call_with_context(context);
        self.second.apply_output(context.into(), output)
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

    struct RoundToInteger;
    impl Callable for RoundToInteger {
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

    struct InitialContext {
        input: i32,
    }

    struct AfterAddOneContext {
        input: i32,
        from_add_one: i32,
    }

    struct AfterMultiplyBy {
        input: i32,
        from_add_one: i32,
        from_multiply_by: i32,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct FinalContext {
        input: i32,
        from_add_one: i32,
        from_multiply_by: i32,
        is_positive: bool,
    }

    impl Context for AddOne {
        type In = InitialContext;
        type Out = AfterAddOneContext;

        fn extract_input(context: &Self::In) -> Self::Input {
            context.input
        }

        fn apply_output(&self, context: Self::In, output: Self::Output) -> Self::Out {
            Self::Out {
                input: context.input,
                from_add_one: output,
            }
        }
    }

    impl Context for MultiplyBy {
        type In = AfterAddOneContext;
        type Out = AfterMultiplyBy;

        fn extract_input(context: &Self::In) -> Self::Input {
            context.from_add_one
        }

        fn apply_output(&self, context: Self::In, from_multiply_by: Self::Output) -> Self::Out {
            let Self::In {
                input,
                from_add_one,
            } = context;
            Self::Out {
                input,
                from_add_one,
                from_multiply_by,
            }
        }
    }

    impl Context for IsPositive {
        type In = AfterMultiplyBy;
        type Out = FinalContext;

        fn extract_input(context: &Self::In) -> Self::Input {
            context.from_multiply_by
        }

        fn apply_output(
            &self,
            AfterMultiplyBy {
                input,
                from_add_one,
                from_multiply_by,
            }: Self::In,
            is_positive: Self::Output,
        ) -> Self::Out {
            Self::Out {
                input,
                from_add_one,
                from_multiply_by,
                is_positive,
            }
        }
    }

    #[test]
    fn chaining_callables() {
        let chain = AddOne
            .then(MultiplyBy { factor: 5 })
            .then(AddOne)
            .then(AddOne);
        assert_eq!(chain.call(7), 42); // (7 + 1) * 5 + 1 + 1 = 42
    }

    #[test]
    fn type_transformations() {
        let chain = AddOne
            .then(ToFloat)
            .then(IncreaseBySmallAmount)
            .then(RoundToInteger);
        assert_eq!(chain.call(3), 4); // 3 + 1 -> 4.0 + 0.1 -> 4

        let boolean_chain = AddOne.then(AddOne).then(IsPositive);
        assert!(boolean_chain.call(-1)); //  -1 + 1 + 1 =  1 (true)
        assert!(!boolean_chain.call(-3)); // -3 + 1 + 1 = -1 (false)
    }

    #[test]
    fn composing_chains() {
        let add_four = AddOne.then(AddOne).then(AddOne).then(AddOne);
        let double_it = MultiplyBy { factor: 2 };

        let chain_one = add_four.then(MultiplyBy { factor: 3 });
        let chain_two = AddOne.then(double_it);

        let composed = chain_one.then(chain_two);

        assert_eq!(composed.call(0), 26); // (((0 + 4) * 3) + 1) * 2 = 26
    }

    #[test]
    fn call_with_context() {
        let multiply_by_three = MultiplyBy { factor: 3 };
        let chain = AddOne.then(multiply_by_three).then(IsPositive);

        assert!(chain.call(1));
        assert!(!chain.call(-3));

        let positive_context = chain.call_with_context(InitialContext { input: 4 });
        assert_eq!(
            positive_context,
            FinalContext {
                input: 4,
                from_add_one: 5,
                from_multiply_by: 15,
                is_positive: true,
            }
        );

        let negative_context = chain.call_with_context(InitialContext { input: -7 });
        assert_eq!(
            negative_context,
            FinalContext {
                input: -7,
                from_add_one: -6,
                from_multiply_by: -18,
                is_positive: false,
            }
        );
    }
}
