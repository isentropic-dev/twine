mod time_integrable;
mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Implements [`TimeIntegrable`] for structs whose fields all implement it.
///
/// When applied to a struct, this macro:
///
/// - Generates a time derivative struct named `{StructName}TimeDerivative`,
///   where each field is a [`TimeDerivative<T>`] corresponding to the original field's type.
/// - Implements [`TimeIntegrable`] for the struct by calling `.step(...)` on each field,
///   using that field's own [`TimeIntegrable`] implementation.
///
/// ## Restrictions
///
/// - The input struct must use named fields (not tuple or unit structs).
/// - All fields must implement [`TimeIntegrable`].
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
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// struct StateVariablesTimeDerivative {
///     temperature: TimeDerivative<ThermodynamicTemperature>,
///     pressure: TimeDerivative<Pressure>,
/// }
///
/// impl TimeIntegrable for StateVariables {
///     type Derivative = StateVariablesTimeDerivative;
///
///     fn step(self, derivative: Self::Derivative, dt: Time) -> Self {
///         Self {
///             temperature: self.temperature.step(derivative.temperature, dt),
///             pressure: self.pressure.step(derivative.pressure, dt),
///         }
///     }
/// }
/// ```
///
/// [`TimeIntegrable`]: twine_core::TimeIntegrable
/// [`TimeDerivative<T>`]: twine_core::TimeDerivative
#[proc_macro_derive(TimeIntegrable)]
pub fn derive_time_integrable(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as time_integrable::Parsed);
    parsed.expand().into()
}
