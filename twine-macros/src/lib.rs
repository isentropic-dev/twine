mod compose;

use proc_macro::TokenStream;

/// Composes multiple components into a single pure function.
///
/// This macro takes a structured definition of inputs and components and
/// generates a module representing the composed component.
///
/// # Example
///
/// ```
/// use twine_macros::compose;
///
/// compose! {
///     my_component {
///         Input {
///             a: f64,
///             b: u32,
///         }
///
///         first => math_component {
///             x: a,
///             y: b * 2,
///         }
///
///         second => math_component {
///             x: first.z,
///             y: 26,
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn compose(input: TokenStream) -> TokenStream {
    compose::expand(input)
}
