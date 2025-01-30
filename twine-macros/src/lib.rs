mod compose;

use proc_macro::TokenStream;

/// Composes multiple components into a higher-order component.
#[proc_macro_attribute]
pub fn compose(attr: TokenStream, item: TokenStream) -> TokenStream {
    compose::expand(attr, item)
}
