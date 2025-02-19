mod compose;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Converts a struct of named components into a generic form with type aliases.
///
/// This macro transforms a struct where each field is a named component,
/// replacing field types with generic parameters. This change ensures that
/// components can be referenced consistently in different contexts, whether
/// as their type, input, or output. It improves composability, preserves LSP
/// support, and generates type aliases for convenient access to the original
/// component types, their inputs, and their outputs.
///
/// ## Behavior
///
/// Given a struct where each field type implements the `twine_core::Component`
/// trait, the `#[compose]` macro:
/// - Retains field names to represent components by name.
/// - Replaces field types with generic parameters.
/// - Generates type aliases for accessing:
///   - `StructNameComponents`: The original component types.
///   - `StructNameInputs`: The input types expected by each component (`Component::Input`).
///   - `StructNameOutputs`: The output types produced by each component (`Component::Output`).
///
/// ## Example
///
/// ### Input
/// ```
/// #[compose]
/// struct Composed {
///     weather: HourlyWeather,
///     building: BuildingModel,
///     solar: SolarArray,
/// }
/// ```
/// ### Expansion
/// ```
/// struct Composed<Weather, Building, Solar> {
///     weather: Weather,
///     building: Building,
///     solar: Solar,
/// }
///
/// type ComposedComponents = Composed<
///     HourlyWeather,
///     BuildingModel,
///     SolarArray
/// >;
///
/// type ComposedInputs = Composed<
///     <HourlyWeather as Component>::Input,
///     <BuildingModel as Component>::Input,
///     <SolarArray as Component>::Input
/// >;
///
/// type ComposedOutputs = Composed<
///     <HourlyWeather as Component>::Output,
///     <BuildingModel as Component>::Output,
///     <SolarArray as Component>::Output
/// >;
/// ```
#[proc_macro_attribute]
pub fn compose(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as compose::Composed);
    parsed.generate_code().into()
}
