use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use super::{ComponentDefinition, ComponentGraph};

pub(crate) fn code(graph: &ComponentGraph) -> TokenStream {
    let definition = &graph.definition;
    let comp_type = &definition.component_type;
    let input_type = &definition.input_type;

    let config_type = generate_aggregate_path(definition, AggregateKind::Config);
    let config_fields = generate_aggregate_fields(definition, AggregateKind::Config);

    let output_type = generate_aggregate_path(definition, AggregateKind::Output);
    let output_fields = generate_aggregate_fields(definition, AggregateKind::Output);

    let create_fn = generate_create_fn(graph);

    let derive_attrs = generate_derive_attributes();

    quote! {
        #derive_attrs
        pub struct #config_type {
            #(#config_fields)*
        }

        #derive_attrs
        pub struct #output_type {
            #(#output_fields)*
        }

        impl twine_core::Component for #comp_type {
            type Config = #config_type;
            type Input = #input_type;
            type Output = #output_type;

            #create_fn
        }
    }
}

/// Appends `Config` or `Output` to the end of a composed component's path.
fn generate_aggregate_path(definition: &ComponentDefinition, kind: AggregateKind) -> Path {
    let mut path = definition.component_type.clone();

    let last = path
        .segments
        .last_mut()
        .expect("Component type path must not be empty.");

    last.ident = format_ident!(
        "{}{}",
        last.ident,
        match kind {
            AggregateKind::Config => "Config",
            AggregateKind::Output => "Output",
        }
    );

    path
}

/// Generates struct fields for `Config` or `Output`.
fn generate_aggregate_fields(
    definition: &ComponentDefinition,
    kind: AggregateKind,
) -> Vec<TokenStream> {
    let field_type = match kind {
        AggregateKind::Config => quote! { Config },
        AggregateKind::Output => quote! { Output },
    };

    definition
        .components
        .iter()
        .map(|c| {
            let name = &c.name;
            let ty = &c.component_type;
            quote! { pub #name: <#ty as twine_core::Component>::#field_type, }
        })
        .collect()
}

/// Generates the `create` function for this composed component.
///
/// This function:
///
/// - Instantiates each component using its configuration from `Config`.
/// - Returns a closure that:
///   - Receives an `Input` struct.
///   - Calls each component in the correct order.
///   - Passes component outputs as inputs to dependent components.
///   - Assembles the computed results into an `Output` struct.
fn generate_create_fn(graph: &ComponentGraph) -> TokenStream {
    let definition = &graph.definition;
    let call_order = graph.call_order();

    // Instantiate each component.
    let instantiate_components: Vec<_> = call_order
        .iter()
        .map(|&index| {
            let component = &definition.components[index];
            let comp_name = &component.name;
            let comp_type = &component.component_type;
            let comp_fn = format_ident!("{}_fn", comp_name);
            quote! {
                let #comp_fn = #comp_type::create(config.#comp_name);
            }
        })
        .collect();

    // Call each component.
    let call_components: Vec<_> = call_order
        .iter()
        .map(|&index| {
            let component = &definition.components[index];
            let comp_name = &component.name;
            let comp_type = &component.component_type;
            let name_fn = format_ident!("{}_fn", comp_name);
            let input_fields = &component.input_struct.fields;

            quote! {
                let #comp_name = #name_fn(<#comp_type as twine_core::Component>::Input { #input_fields });
            }
        })
        .collect();

    // Build output struct.
    let output_fields: Vec<_> = call_order
        .iter()
        .map(|&index| {
            let component = &definition.components[index];
            let name = &component.name;
            quote! { #name, }
        })
        .collect();

    quote! {
        fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
            #(#instantiate_components)*
            move |input| {
                #(#call_components)*
                Self::Output {
                    #(#output_fields)*
                }
            }
        }
    }
}

/// Represents whether we're generating `Config` or `Output`.
#[derive(Clone, Copy)]
enum AggregateKind {
    Config,
    Output,
}

/// Generates `#[derive(...)]` attributes based on feature flags.
fn generate_derive_attributes() -> TokenStream {
    if cfg!(feature = "serde-derive") {
        quote! { #[derive(Debug, Default, serde::Serialize, serde::Deserialize)] }
    } else {
        quote! { #[derive(Debug, Default)] }
    }
}

#[cfg(test)]
mod tests {
    use crate::compose::ComponentInstance;

    use super::*;
    use syn::{parse_quote, punctuated::Punctuated};

    #[test]
    fn generate_aggregate_path_works() {
        let definition = ComponentDefinition {
            component_type: parse_quote! { MyComponent },
            input_type: parse_quote! { MyInput },
            components: vec![],
        };
        assert_eq!(
            generate_aggregate_path(&definition, AggregateKind::Config),
            parse_quote! { MyComponentConfig }
        );
        assert_eq!(
            generate_aggregate_path(&definition, AggregateKind::Output),
            parse_quote! { MyComponentOutput }
        );
    }

    #[test]
    fn generate_aggregate_fields_works() {
        let definition = ComponentDefinition {
            component_type: parse_quote! { MyComponent },
            input_type: parse_quote! { MyInput },
            components: vec![
                ComponentInstance {
                    name: parse_quote! { first },
                    component_type: parse_quote! { ExampleType },
                    input_struct: parse_quote! { ExampleInput { x: 1.0 } },
                },
                ComponentInstance {
                    name: parse_quote! { second },
                    component_type: parse_quote! { AnotherType },
                    input_struct: parse_quote! { AnotherInput { y: 2.0 } },
                },
            ],
        };

        for kind in [AggregateKind::Config, AggregateKind::Output] {
            let generated = generate_aggregate_fields(&definition, kind);
            let expected = match kind {
                AggregateKind::Config => quote! {
                    pub first: <ExampleType as twine_core::Component>::Config,
                    pub second: <AnotherType as twine_core::Component>::Config,
                },
                AggregateKind::Output => quote! {
                    pub first: <ExampleType as twine_core::Component>::Output,
                    pub second: <AnotherType as twine_core::Component>::Output,
                },
            };

            assert_eq!(quote! { #(#generated)* }.to_string(), expected.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "Component type path must not be empty.")]
    fn generate_aggregate_path_panics_on_empty_path() {
        let definition = ComponentDefinition {
            component_type: Path {
                leading_colon: None,
                segments: Punctuated::new(),
            },
            input_type: parse_quote! { MyInput },
            components: vec![],
        };

        let _ = generate_aggregate_path(&definition, AggregateKind::Config);
    }

    #[test]
    fn generate_create_fn_works() {
        let definition = ComponentDefinition {
            component_type: parse_quote! { MyComponent },
            input_type: parse_quote! { MyInput },
            components: vec![
                ComponentInstance {
                    name: parse_quote! { call_second },
                    component_type: parse_quote! { ExampleType },
                    input_struct: parse_quote! { ExampleInput { x: call_first.z } },
                },
                ComponentInstance {
                    name: parse_quote! { call_first },
                    component_type: parse_quote! { AnotherType },
                    input_struct: parse_quote! { AnotherInput { y: 2.0 } },
                },
            ],
        };
        let graph = definition.into();
        let generated = generate_create_fn(&graph);
        let expected = quote! {
            pub fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
                let call_first_fn = AnotherType::create(config.call_first);
                let call_second_fn = ExampleType::create(config.call_second);

                move |input| {
                    let call_first = call_first_fn(<AnotherType as twine_core::Component>::Input { y: 2.0 });
                    let call_second = call_second_fn(<ExampleType as twine_core::Component>::Input { x: call_first.z });
                }
            }
        };
        assert_eq!(quote! { #generated }.to_string(), expected.to_string());
    }
}
