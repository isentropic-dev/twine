use proc_macro2::TokenStream;

use super::ComponentGraph;

pub(crate) fn code(_graph: &ComponentGraph) -> TokenStream {
    // For now we won't generate any code...
    TokenStream::new()
}
