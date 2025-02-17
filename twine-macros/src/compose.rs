use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Attribute, Error, Field, Fields, FieldsNamed, Ident, ItemStruct, Result,
    Visibility,
};

/// Represents a struct processed by the `#[compose]` macro.
#[derive(Debug)]
pub(crate) struct Composed {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    fields: FieldsNamed,
}

impl Parse for Composed {
    /// Parses a struct definition from the input stream.
    ///
    /// The struct must have named fields (i.e., it can't be a tuple or unit
    /// struct) and cannot use generics. The `#[compose]` attribute is removed,
    /// but all other attributes and visibility modifiers remain unchanged.
    fn parse(input: ParseStream) -> Result<Self> {
        let ItemStruct {
            attrs,
            vis,
            ident,
            generics,
            fields,
            ..
        } = input.parse()?;

        let Fields::Named(fields) = fields else {
            return Err(Error::new_spanned(
                ident,
                "Unsupported struct type. This macro requires a struct with named fields.",
            ));
        };

        if !generics.params.is_empty() {
            return Err(Error::new_spanned(
                generics,
                "Generic parameters are not allowed. Remove them to use this macro.",
            ));
        }

        let attrs: Vec<Attribute> = attrs
            .into_iter()
            .filter(|attr| !attr.path().is_ident("compose"))
            .collect();

        Ok(Composed {
            attrs,
            vis,
            ident,
            fields,
        })
    }
}

impl Composed {
    /// Generates a generic version of the parsed struct with type aliases.
    ///
    /// The generated struct keeps its name but replaces field types with
    /// generic parameters. Additionally, three type aliases are created for
    /// specific versions of the generic struct:
    ///
    /// - `StructNameComponents`
    ///   - Uses the original field types.
    ///
    /// - `StructNameInputs`
    ///   - Uses `<FieldType as Component>::Input`.
    ///   - Represents the types each component expects as input.
    ///
    /// - `StructNameOutputs`
    ///   - Uses `<FieldType as Component>::Output`.
    ///   - Represents the types each component produces as output.
    pub fn generate_code(self) -> TokenStream {
        let Self {
            attrs,
            vis,
            ident,
            mut fields,
        } = self;

        let components_alias = format_ident!("{}Components", ident);
        let inputs_alias = format_ident!("{}Inputs", ident);
        let outputs_alias = format_ident!("{}Outputs", ident);

        let original_types: Vec<_> = fields
            .named
            .iter()
            .map(|Field { ty, .. }| {
                quote! { #ty }
            })
            .collect();

        let generic_params: Vec<_> = fields
            .named
            .iter()
            .map(|field| {
                let param_name = field
                    .ident
                    .as_ref()
                    .expect("Identifier exists because parsing ensures named fields")
                    .to_string()
                    .to_upper_camel_case();
                format_ident!("{param_name}")
            })
            .collect();

        for (field, generic_ident) in fields.named.iter_mut().zip(&generic_params) {
            field.ty = parse_quote! { #generic_ident };
        }

        let input_types = original_types
            .iter()
            .map(|ty| quote! { <#ty as Component>::Input });

        let output_types = original_types
            .iter()
            .map(|ty| quote! { <#ty as Component>::Output });

        quote! {
            #(#attrs)*
            #vis struct #ident <#(#generic_params),*> #fields

            #vis type #components_alias = #ident<#(#original_types),*>;
            #vis type #inputs_alias = #ident<#(#input_types),*>;
            #vis type #outputs_alias = #ident<#(#output_types),*>;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_str;

    #[test]
    fn generates_correct_code() {
        let input = "
            pub struct MyComponent {
                add_one: Adder<f64>,
                pub(super) add_two: Adder<f64>,
                pub(crate) math: Arithmetic,
            }
        ";

        let composed: Composed = parse_str(input).expect("Parsing should succeed");
        let generated = composed.generate_code();

        let expected = quote! {
            pub struct MyComponent<AddOne, AddTwo, Math> {
                add_one: AddOne,
                pub(super) add_two: AddTwo,
                pub(crate) math: Math,
            }

            pub type MyComponentComponents = MyComponent<Adder<f64>, Adder<f64>, Arithmetic>;

            pub type MyComponentInputs = MyComponent<
                <Adder<f64> as Component>::Input,
                <Adder<f64> as Component>::Input,
                <Arithmetic as Component>::Input
            >;

            pub type MyComponentOutputs = MyComponent<
                <Adder<f64> as Component>::Output,
                <Adder<f64> as Component>::Output,
                <Arithmetic as Component>::Output
            >;
        };

        assert_eq!(generated.to_string(), expected.to_string());
    }

    #[test]
    fn rejects_tuple_struct() {
        let input = "struct TupleComponent(i32, f64);";

        let err = parse_str::<Composed>(input).expect_err("Parsing should fail");

        assert!(
            err.to_string().contains("Unsupported struct type"),
            "Unexpected error message: {err}"
        );
    }

    #[test]
    fn rejects_generic_struct() {
        let input = "
            struct GenericComponent<T> {
                field: T,
            }
        ";

        let err = parse_str::<Composed>(input).expect_err("Parsing should fail");

        assert!(
            err.to_string()
                .contains("Generic parameters are not allowed"),
            "Unexpected error message: {err}"
        );
    }
}
