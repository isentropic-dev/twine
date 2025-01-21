mod compose;

use proc_macro::TokenStream;

/// Composes multiple components into a higher-order component.
///
/// This macro generates a module representing the composed component based on a
/// structured definition of inputs and components. It simplifies the process of
/// wiring inputs, outputs, and dependencies between components.
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
///             y: b * 6,
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
