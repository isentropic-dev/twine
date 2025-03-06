mod composable;
mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Converts a struct of named components into a composable template.
///
/// This macro transforms a struct that defines component types into a generic
/// template while preserving those types via an associated trait. It enables
/// type-safe composition by generating a trait-based mapping of field types
/// and implementing `twine_core::Composable` for the generic struct with type
/// parameters corresponding to the original component types.
///
/// ## What This Macro Does
///
/// - Replaces the original struct with a generic `{StructName}`.
/// - Generates `{StructName}Types`, a trait that exposes original field types.
/// - Implements `{StructName}Types` for `()`, enabling type lookup from the trait alone.
/// - Implements `twine_core::Composable` on the generic version of the struct, defining:
///   - `Inputs`: `{StructName}` where fields use `<CompType as Component>::Input`.
///   - `Outputs`: `{StructName}` where fields use `<CompType as Component>::Output`.
///
/// ## Naming Conventions
///
/// - `{StructName}`: The transformed generic struct.
/// - `{StructName}Types`: The trait that maps field names to types.
/// - Generic parameters and the associated types in `{StructName}Types` use the
///   corresponding field names, converted to `UpperCamelCase`.
///
/// ## Usage
///
/// `{StructName}Types` allows accessing individual component types generically:
///
/// ```ignore
/// type AddOneType = <() as MyComponentsTypes>::AddOne;
/// let _x: AddOneType = Adder::new(1);
///
/// // Access the fully composed struct.
/// type ConcreteAlias = <() as MyComponentsTypes>::__Alias;
/// let instance: ConcreteAlias = MyComponents {
///     add_one: Adder::new(1),
///     add_two: Adder::new(2),
///     math: Arithmetic,
/// };
/// ```
///
/// ## Restrictions
///
/// - The struct must use named fields.
/// - All field types must implement `twine_core::Component`.
/// - Attributes other than documentation comments are not allowed.
/// - Generic parameters cannot be used.
///
/// ## Example
///
/// ### Before
/// ```
/// #[composable]
/// pub struct MyComponents {
///     pub add_one: Adder<f64>,
///     pub add_two: Adder<f64>,
///     pub math: Arithmetic,
/// }
/// ```
///
/// ### After Macro Expansion
/// ```
/// pub struct MyComponents<AddOne, AddTwo, Math> {
///     pub add_one: AddOne,
///     pub add_two: AddTwo,
///     pub math: Math,
/// }
///
/// pub trait MyComponentsTypes {
///     type __Alias;
///     type AddOne;
///     type AddTwo;
///     type Math;
/// }
///
/// impl MyComponentsTypes for () {
///     type __Alias = MyComponents<Adder<f64>, Adder<f64>, Arithmetic>;
///     type AddOne = Adder<f64>;
///     type AddTwo = Adder<f64>;
///     type Math = Arithmetic;
/// }
///
/// impl twine_core::Composable for MyComponents<Adder<f64>, Adder<f64>, Arithmetic> {
///     type Inputs = MyComponents<
///         <Adder<f64> as twine_core::Component>::Input,
///         <Adder<f64> as twine_core::Component>::Input,
///         <Arithmetic as twine_core::Component>::Input
///     >;
///
///     type Outputs = MyComponents<
///         <Adder<f64> as twine_core::Component>::Output,
///         <Adder<f64> as twine_core::Component>::Output,
///         <Arithmetic as twine_core::Component>::Output
///     >;
/// }
/// ```
#[proc_macro_attribute]
pub fn composable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as composable::Parsed);
    parsed.generate_code().into()
}
