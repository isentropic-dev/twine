use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Token, Type,
};

#[allow(dead_code)]
#[derive(Debug)]
struct ComponentDefinition {
    name: NameSection,
    inputs: InputsSection,
}

#[derive(Debug)]
struct NameSection {
    name: Ident,
}

#[allow(dead_code)]
#[derive(Debug)]
struct InputsSection {
    fields: Vec<(Ident, Type)>,
}

impl Parse for ComponentDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        let inputs = input.parse()?;

        Ok(Self { name, inputs })
    }
}

/// Parse the `name: foo` line.
impl Parse for NameSection {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword: Ident = input.parse()?;
        if keyword != "name" {
            return Err(syn::Error::new_spanned(keyword, "expected `name:`"));
        }
        input.parse::<Token![:]>()?;

        // Parse whatever token comes next.
        let potential_name: Ident = match input.parse::<Ident>() {
            Ok(id) => id,
            Err(e) => {
                return Err(syn::Error::new(e.span(), "expected a valid component name"));
            }
        };

        // Check that the next token isn't the start of the inputs section,
        // which would indicate a name wasn't actually provided.
        if potential_name == "inputs" && input.peek(Token![:]) {
            return Err(syn::Error::new_spanned(
                potential_name,
                "expected a component name, but found `inputs:` (start of next section)",
            ));
        }

        Ok(NameSection {
            name: potential_name,
        })
    }
}

/// Parse the `inputs:` section.
impl Parse for InputsSection {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword: Ident = input.parse()?;
        if keyword != "inputs" {
            return Err(syn::Error::new_spanned(keyword, "expected `inputs:`"));
        }
        input.parse::<Token![:]>()?;

        // Read line by line until we get to the `components:` section.
        let mut fields = Vec::new();
        while !input.is_empty() {
            // Break if the next tokens match `Ident("components") :`.
            if input.peek(Ident) && input.peek2(Token![:]) {
                // Fork so we don't consume these tokens in our real stream.
                let forked = input.fork();
                let ident: Ident = forked.parse()?;
                if ident == "components" {
                    break;
                }
            }

            // Parse this input's name.
            let field_name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            // Check if a type wasn't provided.
            if input.is_empty() || (input.peek(Ident) && input.peek2(Token![:])) {
                return Err(syn::Error::new(
                    field_name.span(),
                    "expected a type for this input",
                ));
            }

            // Otherwise, parse the type normally.
            let field_type: Type = input.parse()?;
            fields.push((field_name, field_type));
        }

        Ok(InputsSection { fields })
    }
}

#[proc_macro]
pub fn define_component(input: TokenStream) -> TokenStream {
    let ComponentDefinition { name, inputs: _ } = parse_macro_input!(input as ComponentDefinition);
    let module_name = name.name;

    let generated_code = quote! {
        pub(crate) mod #module_name {
            pub(crate) struct Config;
            pub(crate) struct Input;
            pub(crate) struct Output;
        }
    };

    TokenStream::from(generated_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, parse_str};

    fn assert_error_message<T: syn::parse::Parse + std::fmt::Debug>(
        input: &str,
        expected_msg: &str,
    ) {
        let err = syn::parse_str::<T>(input).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(expected_msg),
            "Got unexpected error message: {msg}"
        );
    }

    #[test]
    fn parse_name_succeeds() {
        let input = "name: test_component";
        let parsed = parse_str::<NameSection>(input).unwrap();
        assert_eq!(parsed.name, "test_component");
    }

    #[test]
    fn parse_name_fails_with_bad_input() {
        assert_error_message::<NameSection>(
            "
            nam: test_component
            ",
            "expected `name:`",
        );

        assert_error_message::<NameSection>(
            "
            name: 42
            ",
            "expected a valid component name",
        );

        assert_error_message::<NameSection>(
            "
            name:
            inputs:
            ",
            "expected a component name, but found `inputs:` (start of next section)",
        );
    }

    #[test]
    fn parse_inputs_section_succeeds() {
        let input = "
            inputs:
                first_input: i32
                second_input: f64
        ";
        let parsed = parse_str::<InputsSection>(input).unwrap();
        assert_eq!(parsed.fields.len(), 2);

        assert_eq!(parsed.fields[0].0, "first_input");
        assert_eq!(parsed.fields[0].1, parse_quote!(i32));

        assert_eq!(parsed.fields[1].0, "second_input");
        assert_eq!(parsed.fields[1].1, parse_quote!(f64));
    }

    #[test]
    fn parse_inputs_section_fails_with_bad_input() {
        assert_error_message::<InputsSection>(
            "
            inputss:
                something: i32
            ",
            "expected `inputs:`",
        );

        assert_error_message::<InputsSection>(
            "
            inputs:
                missing_first:
                something: i32
            ",
            "expected a type for this input",
        );

        assert_error_message::<InputsSection>(
            "
            inputs:
                something: i32
                missing_last:
            ",
            "expected a type for this input",
        );

        assert_error_message::<InputsSection>(
            "
            inputs:
                something: i32
                missing_middle:
                last_input: f64
            ",
            "expected a type for this input",
        );
    }
}
