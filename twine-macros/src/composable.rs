use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Error, Fields, FieldsNamed, Ident, ItemStruct, Result, Type, Visibility,
    parse::{Parse, ParseStream},
    parse_quote,
};

use crate::utils::IdentExt;

#[derive(Debug)]
pub(crate) struct Parsed {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    fields: FieldsNamed,
}

impl Parse for Parsed {
    /// Parses a struct definition and validates constraints.
    fn parse(input: ParseStream) -> Result<Self> {
        let ItemStruct {
            attrs,
            vis,
            ident,
            generics,
            fields,
            ..
        } = input.parse()?;

        if attrs.iter().any(|attr| !attr.path().is_ident("doc")) {
            return Err(Error::new_spanned(
                ident,
                r"Only doc attributes (`///`) are allowed. Remove other attributes to use this macro.",
            ));
        }

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

        Ok(Parsed {
            attrs,
            vis,
            ident,
            fields,
        })
    }
}

impl Parsed {
    /// Generates the full token stream for the macro expansion.
    pub fn expand(self) -> TokenStream {
        let generic_struct = self.generate_generic_struct();
        let types_trait = self.generate_types_trait();

        quote! {
            #generic_struct
            #types_trait
        }
    }

    /// Generates a generic struct with type parameters for each field.
    fn generate_generic_struct(&self) -> TokenStream {
        let Self {
            attrs,
            vis,
            ident,
            fields,
        } = self;

        let generic_params: Vec<_> = self.iter_fields_as_generics().collect();

        let generic_fields: Vec<_> = fields
            .named
            .iter()
            .zip(&generic_params)
            .map(|(field, generic_param)| {
                let mut field = field.clone();
                field.ty = parse_quote! { #generic_param };
                field
            })
            .collect();

        quote! {
            #(#attrs)*
            #vis struct #ident<#(#generic_params),*> {
                #(#generic_fields),*
            }
        }
    }

    /// Generates a trait that exposes the original field types.
    fn generate_types_trait(&self) -> TokenStream {
        let Self { vis, ident, .. } = self;

        let trait_name = ident.with_suffix("Types");
        let trait_doc = format!("Provides access to the original field types of `{ident}`.");

        let field_types = self.iter_fields_as_generics().map(|generic_param| {
            quote! { type #generic_param; }
        });

        let comp_types = self.iter_fields_as_types().map(|comp_type| {
            quote! { #comp_type }
        });

        let input_types = self.iter_fields_as_types().map(|comp_type| {
            quote! { <#comp_type as twine_core::Component>::Input }
        });

        let output_types = self.iter_fields_as_types().map(|comp_type| {
            quote! { <#comp_type as twine_core::Component>::Output}
        });

        let impl_associated_types = self
            .iter_fields_as_generics()
            .zip(self.iter_fields_as_types())
            .map(|(generic_param, comp_type)| {
                quote! { type #generic_param = #comp_type; }
            });

        quote! {
            #[doc = #trait_doc]
            #vis trait #trait_name {
                type __Concrete;
                type __Inputs;
                type __Outputs;
                #(#field_types)*
            }

            impl #trait_name for () {
                type __Concrete = #ident<#(#comp_types),*>;
                type __Inputs = #ident<#(#input_types),*>;
                type __Outputs = #ident<#(#output_types),*>;
                #(#impl_associated_types)*
            }
        }
    }

    /// Iterates over struct fields and generates generic type names.
    fn iter_fields_as_generics(&self) -> impl Iterator<Item = Ident> {
        self.fields.named.iter().map(|field| {
            field
                .ident
                .as_ref()
                .expect("Identifiers always exist for named fields")
                .upper_camel_case()
        })
    }

    /// Iterates over struct fields and returns their original types.
    fn iter_fields_as_types(&self) -> impl Iterator<Item = Type> {
        self.fields.named.iter().map(|field| field.ty.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_str;

    #[test]
    fn generates_correct_code() {
        let input = "
            /// The components in my model.
            pub struct MyComponents {
                add_one: Adder<f64>,
                pub(super) add_two: Adder<f64>,
                pub(crate) math: Arithmetic,
            }
        ";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[doc = " The components in my model."]
            pub struct MyComponents<AddOne, AddTwo, Math> {
                add_one: AddOne,
                pub(super) add_two: AddTwo,
                pub(crate) math: Math
            }

            #[doc = "Provides access to the original field types of `MyComponents`."]
            pub trait MyComponentsTypes {
                type __Concrete;
                type __Inputs;
                type __Outputs;
                type AddOne;
                type AddTwo;
                type Math;
            }

            impl MyComponentsTypes for () {
                type __Concrete = MyComponents<Adder<f64>, Adder<f64>, Arithmetic>;
                type __Inputs = MyComponents<
                    <Adder<f64> as twine_core::Component>::Input,
                    <Adder<f64> as twine_core::Component>::Input,
                    <Arithmetic as twine_core::Component>::Input
                >;
                type __Outputs = MyComponents<
                    <Adder<f64> as twine_core::Component>::Output,
                    <Adder<f64> as twine_core::Component>::Output,
                    <Arithmetic as twine_core::Component>::Output
                >;
                type AddOne = Adder<f64>;
                type AddTwo = Adder<f64>;
                type Math = Arithmetic;
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn error_if_attributes_are_present() {
        let error_message = parse_str::<Parsed>(
            "#[derive(Debug)]
            struct ComponentsWithAttributes {
                comp: SomeComp,
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error_message,
            "Only doc attributes (`///`) are allowed. Remove other attributes to use this macro."
        );
    }

    #[test]
    fn error_if_tuple_struct() {
        let error_message = parse_str::<Parsed>("struct TupleComponents(i32, f64);")
            .unwrap_err()
            .to_string();

        assert_eq!(
            error_message,
            "Unsupported struct type. This macro requires a struct with named fields."
        );
    }

    #[test]
    fn error_if_generics_are_present() {
        let error_message = parse_str::<Parsed>(
            "struct ComponentsWithGenerics<T> {
                comp: T,
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error_message,
            "Generic parameters are not allowed. Remove them to use this macro."
        );
    }
}
