use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Token,
};

struct ComponentDefinition {
    name: String,
}

impl Parse for ComponentDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the `name: component_name` line.
        let _name_keyword: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let name_ident: Ident = input.parse()?;
        Ok(Self {
            name: name_ident.to_string(),
        })
    }
}

#[proc_macro]
pub fn define_component(input: TokenStream) -> TokenStream {
    // Parse the macro input.
    let ComponentDefinition { name } = parse_macro_input!(input as ComponentDefinition);

    // Generate the code.
    let module_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
    let generated_code = quote! {
        pub(crate) mod #module_name {
            pub(crate) struct Config;
            pub(crate) struct Input;
            pub(crate) struct Output;
        }
    };

    // Return the generated code.
    TokenStream::from(generated_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn parse_component_definition() {
        let parsed: ComponentDefinition = parse_str(
            r"
                name: system
            ",
        )
        .unwrap();
        assert_eq!(parsed.name, "system");

        let parsed: ComponentDefinition = parse_str(
            r"
                name: example
            ",
        )
        .unwrap();
        assert_eq!(parsed.name, "example");
    }
}
