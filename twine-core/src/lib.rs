#[cfg(feature = "macros")]
pub use twine_macros::compose;

/// The fundamental trait for defining components in Twine.
///
/// The `Component` trait is the core abstraction in Twine, representing any
/// unit that can be composed into a larger system. A component is a pure
/// function that maps its input to its output and is configured with static
/// parameters at creation.
///
/// Any type that implements `Component` can be composed in Twine.
///
/// Components are instantiated through a factory function:
///
/// ```ignore
/// fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output;
/// ```
///
/// The returned function must be pure, meaning it has no side effects and can
/// be safely composed into larger systems where Twine manages dependencies
/// and execution.
///
/// # Implementing a Component
///
/// To define a component, a type must implement the `Component` trait by
/// specifying its configuration, input, and output types, and providing a
/// `create` function that returns a pure function mapping input to output.
///
/// # Example
///
/// ```rust
/// mod example {
///     use twine_core::Component;
///
///     struct MultiplierConfig {
///         factor: f64,
///     }
///
///     struct MultiplierInput {
///         value: f64,
///     }
///
///     struct MultiplierOutput {
///         result: f64,
///     }
///
///     struct Multiplier;
///
///     impl Component for Multiplier {
///         type Config = MultiplierConfig;
///         type Input = MultiplierInput;
///         type Output = MultiplierOutput;
///
///         fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
///             move |input| MultiplierOutput {
///                 result: input.value * config.factor,
///             }
///         }
///     }
/// }
/// ```
///
/// Components like `Multiplier` can be composed declaratively using `compose!`:
///
/// ```ignore
/// compose!(chained_multiplier, {
///     Input {
///         start_from: f64,
///     }
///     first => example::Multiplier { value: start_from }
///     second => example::Multiplier { value: first.result }
///     third => example::Multiplier { value: second.result }
/// });
/// ```
pub trait Component {
    type Config;
    type Input;
    type Output;

    fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output;
}
