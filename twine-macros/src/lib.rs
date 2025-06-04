mod composable;
mod compose;
mod time_integrable;
mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Converts a struct of named components into a composable template.
///
/// This macro simplifies component-based model creation by transforming a
/// struct into a generic template and automatically generating a trait that
/// defines associated type mappings. This approach facilitates type-safe and
/// flexible composition of components.
///
/// When applied to a struct, this macro:
///
/// - Replaces the original struct with a generic version (`{StructName}`).
/// - Generates a trait (`{StructName}Types`) that preserves the original field
///   types and provides aliases to common struct variants.
///
/// ## Naming Conventions
///
/// - `{StructName}`: The generic version of the struct.
/// - `{StructName}Types`: A trait exposing:
///   - Associated types for each original field.
///   - Additional type aliases:
///     - `__Concrete`: The original struct type with concrete component types.
///     - `__Inputs`: A struct variant with fields using `<CompType as Component>::Input`.
///     - `__Outputs`: A struct variant with fields using `<CompType as Component>::Output`.
///
/// Field names are transformed into `UpperCamelCase` for generic parameters and
/// associated types.
///
/// ## Restrictions
///
/// - Structs must use named fields.
/// - Field types must implement `twine_core::Component`.
/// - Only documentation attributes (`///`) are permitted.
/// - Generic parameters are not supported.
///
/// ## Types Trait Usage
///
/// Access original field types generically:
///
/// ```ignore
/// type AddOneType = <() as MyComponentsTypes>::AddOne;
/// let adder: AddOneType = Adder::new(1);
///
/// // Access the concrete composed struct.
/// type Concrete = <() as MyComponentsTypes>::__Concrete;
/// let components: Concrete = MyComponents {
///     add_one: Adder::new(1),
///     add_two: Adder::new(2),
///     math: Arithmetic,
/// };
/// ```
///
/// ## Example
///
/// ### Input
///
/// ```ignore
/// #[composable]
/// pub struct MyComponents {
///     pub add_one: Adder<f64>,
///     pub add_two: Adder<f64>,
///     pub math: Arithmetic,
/// }
/// ```
///
/// ### Expanded
///
/// ```ignore
/// pub struct MyComponents<AddOne, AddTwo, Math> {
///     pub add_one: AddOne,
///     pub add_two: AddTwo,
///     pub math: Math,
/// }
///
/// pub trait MyComponentsTypes {
///     type __Concrete;
///     type __Inputs;
///     type __Outputs;
///     type AddOne;
///     type AddTwo;
///     type Math;
/// }
///
/// impl MyComponentsTypes for () {
///     type __Concrete = MyComponents<
///         Adder<f64>,
///         Adder<f64>,
///         Arithmetic
///     >;
///
///     type __Inputs = MyComponents<
///         <Adder<f64> as twine_core::Component>::Input,
///         <Adder<f64> as twine_core::Component>::Input,
///         <Arithmetic as twine_core::Component>::Input
///     >;
///
///     type __Outputs = MyComponents<
///         <Adder<f64> as twine_core::Component>::Output,
///         <Adder<f64> as twine_core::Component>::Output,
///         <Arithmetic as twine_core::Component>::Output
///     >;
///
///     type AddOne = Adder<f64>;
///     type AddTwo = Adder<f64>;
///     type Math = Arithmetic;
/// }
/// ```
#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as composable::Parsed);
    parsed.expand().into()
}

/// Creates a component from multiple `#[composable]` components.
///
/// This macro generates a concrete struct implementing `twine_core::Component`.
/// The generated component wraps an internal `twine_core::Twine` chain,
/// automatically managing component execution order based on user-specified
/// connections between the top-level input and component outputs.
///
/// ## Restrictions
///
/// - Exactly two type aliases must be defined:
///   - `Input`: The top-level input type.
///   - `Components`: References a struct previously defined with `#[composable]`.
/// - Connection expressions may only reference fields from:
///   - The top-level `input`, or
///   - Outputs from other components (`output.{component_name}`).
/// - All referenced components must implement `twine_core::Component`.
/// - Cyclic dependencies between components are not currently permitted.
///
/// ## Example
///
/// ```ignore
/// #[composable]
/// pub struct CalcComponents {
///     pub adder: Adder<i32>,
///     pub multiplier: Multiplier<i32>,
/// }
///
/// pub struct CalcInput {
///     value: i32,
/// }
///
/// #[compose(Calculator)]
/// fn compose() {
///     type Input = CalcInput;
///     type Components = CalcComponents;
///
///     Connections {
///         adder: input.value,
///         multiplier: output.adder,
///     }
/// }
///
/// let calculator = Calculator::new(CalcComponents {
///     adder: Adder::new(1),
///     multiplier: Multiplier::new(2),
/// });
///
/// let result = calculator.call(CalcInput { value: 10 }).unwrap();
/// ```
#[proc_macro_attribute]
pub fn compose(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = compose::Parsed::new(
        parse_macro_input!(attr as compose::ParsedAttr),
        parse_macro_input!(item as compose::ParsedItem),
    );
    parsed.try_expand().unwrap_or_else(|err| err).into()
}

/// Derives the `TimeIntegrable` trait for structs containing state variables.
///
/// This macro automates the implementation of time integration for simulation
/// state structs by generating the necessary boilerplate code for numerical
/// integration. It creates a derivatives struct and implements the required
/// traits for time-stepping operations.
///
/// When applied to a struct, this macro:
///
/// - Generates a derivatives struct (`{StructName}Dt`) with `TimeDerivativeOf<T>`
///   fields corresponding to each original field.
/// - Implements `Div<Time>` to convert state variables to their time derivatives.
/// - Implements `TimeIntegrable` to perform time-stepping integration.
///
/// ## Naming Conventions
///
/// - `{StructName}Dt`: A struct containing time derivatives of each field,
///   with field names suffixed with `_dt`.
///
/// ## Restrictions
///
/// - Structs must use named fields.
/// - Generic parameters are not supported.
/// - All field types must support division by `Time` and addition operations
///   required for integration (the compiler will enforce these constraints).
///
/// ## Integration Pattern
///
/// The generated code follows this mathematical pattern for each field:
///
/// ```text
/// new_value = old_value + derivative * dt
/// ```
///
/// Where `derivative` comes from dividing the field by time, and `dt` is the
/// integration time step.
///
/// ## Example
///
/// ### Input
///
/// ```ignore
/// #[derive(TimeIntegrable)]
/// struct StateVariables {
///     temperature: ThermodynamicTemperature,
///     pressure: Pressure,
/// }
/// ```
///
/// ### Expanded
///
/// ```ignore
/// struct StateVariablesDt {
///     temperature_dt: TimeDerivativeOf<ThermodynamicTemperature>,
///     pressure_dt: TimeDerivativeOf<Pressure>,
/// }
///
/// impl Div<Time> for StateVariables {
///     type Output = StateVariablesDt;
///
///     fn div(self, rhs: Time) -> Self::Output {
///         Self::Output {
///             temperature_dt: self.temperature / rhs,
///             pressure_dt: self.pressure / rhs,
///         }
///     }
/// }
///
/// impl TimeIntegrable for StateVariables {
///     fn step_by_time(self, derivative: StateVariablesDt, dt: Time) -> Self {
///         Self {
///             temperature: self.temperature + derivative.temperature_dt * dt,
///             pressure: self.pressure + derivative.pressure_dt * dt,
///         }
///     }
/// }
/// ```
#[proc_macro_derive(TimeIntegrable)]
pub fn derive_time_integrable(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as time_integrable::Parsed);
    parsed.expand().into()
}
