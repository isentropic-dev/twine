use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote_spanned, Ident, Path};

use super::{ComponentDefinition, ComponentGraph, ComponentInstance};

pub(crate) fn code(graph: &ComponentGraph) -> TokenStream {
    let definition = &graph.definition;
    let comp_type = &definition.component_type;
    let input_type = &definition.input_type;

    let config_type = generate_aggregate_path(definition, AggregateKind::Config);
    let config_fields = generate_aggregate_fields(definition, AggregateKind::Config);

    let output_type = generate_aggregate_path(definition, AggregateKind::Output);
    let output_fields = generate_aggregate_fields(definition, AggregateKind::Output);

    let derive_attrs = generate_derive_attributes();
    let create_fn = generate_create_fn(graph);

    quote! {
        #derive_attrs
        pub struct #config_type {
            #(#config_fields)*
        }

        #derive_attrs
        pub struct #output_type {
            #(#output_fields)*
        }

        impl twine_core::legacy::Component for #comp_type {
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
            quote! { pub #name: <#ty as twine_core::legacy::Component>::#field_type, }
        })
        .collect()
}

/// Generates the `create` function for this composed component.
///
/// This function performs the following:
///
/// - Defines type aliases to handle Rust's qualified path limitations.
/// - Instantiates each component using its configuration from `Config`.
/// - Returns a closure that:
///   - Accepts an `Input` struct.
///   - Calls each component in the correct order.
///   - Passes outputs as inputs to dependent components.
///   - Aggregates results into an `Output` struct.
fn generate_create_fn(graph: &ComponentGraph) -> TokenStream {
    let input_type_aliases = generate_input_type_aliases(graph);
    let instantiate_components = generate_instantiate_components(graph);
    let call_components = generate_call_components(graph);
    let output_struct = generate_output_struct(graph);

    quote! {
        fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
            #(#input_type_aliases)*
            #(#instantiate_components)*
            move |input| {
                #(#call_components)*
                #output_struct
            }
        }
    }
}

/// Generates type aliases for component inputs.
fn generate_input_type_aliases(graph: &ComponentGraph) -> Vec<TokenStream> {
    let mut seen_types = HashSet::new();
    graph
        .iter_components()
        .filter_map(|ComponentInstance { component_type, .. }| {
            let input_alias = input_alias_for(component_type);
            if seen_types.insert(input_alias.to_string()) {
                Some(quote! {
                    type #input_alias = <#component_type as twine_core::legacy::Component>::Input;
                })
            } else {
                None
            }
        })
        .collect()
}

/// Instantiates each component using its configuration.
fn generate_instantiate_components(graph: &ComponentGraph) -> Vec<TokenStream> {
    graph
        .iter_components()
        .map(
            |ComponentInstance {
                 name,
                 component_type,
                 ..
             }| {
                let component_fn = format_ident!("{name}_fn");
                quote! {
                    let #component_fn = #component_type::create(config.#name);
                }
            },
        )
        .collect()
}

/// Calls each component in the correct order.
fn generate_call_components(graph: &ComponentGraph) -> Vec<TokenStream> {
    graph
        .iter_components()
        .map(
            |ComponentInstance {
                 name,
                 component_type,
                 input_struct,
             }| {
                // This bit allows rust-analyzer to work as expected.
                let mut input_expr = input_struct.clone();
                let input_alias = input_alias_for(component_type);
                input_expr.path = parse_quote_spanned!(Span::call_site() => #input_alias);

                let component_fn = format_ident!("{name}_fn");
                quote! {
                    let #name = #component_fn(#input_expr);
                }
            },
        )
        .collect()
}

/// Constructs the `Self::Output` struct with computed values.
fn generate_output_struct(graph: &ComponentGraph) -> TokenStream {
    let output_fields = graph
        .iter_components()
        .map(|ComponentInstance { name, .. }| {
            quote! { #name, }
        });
    quote! {
        Self::Output { #(#output_fields)* }
    }
}

/// Derives the input type alias for a given component type.
fn input_alias_for(component_type: &Path) -> Ident {
    let last_segment = component_type
        .segments
        .last()
        .expect("Component type path must not be empty");
    format_ident!("{}Input", last_segment.ident)
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
                    pub first: <ExampleType as twine_core::legacy::Component>::Config,
                    pub second: <AnotherType as twine_core::legacy::Component>::Config,
                },
                AggregateKind::Output => quote! {
                    pub first: <ExampleType as twine_core::legacy::Component>::Output,
                    pub second: <AnotherType as twine_core::legacy::Component>::Output,
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
            fn create(config: Self::Config) -> impl Fn(Self::Input) -> Self::Output {
                type AnotherTypeInput = <AnotherType as twine_core::legacy::Component>::Input;
                type ExampleTypeInput = <ExampleType as twine_core::legacy::Component>::Input;
                let call_first_fn = AnotherType::create(config.call_first);
                let call_second_fn = ExampleType::create(config.call_second);

                move |input| {
                    let call_first = call_first_fn(AnotherTypeInput { y: 2.0 });
                    let call_second = call_second_fn(ExampleTypeInput { x: call_first.z });
                    Self::Output {
                        call_first,
                        call_second,
                    }
                }
            }
        };
        assert_eq!(quote! { #generated }.to_string(), expected.to_string());
    }
}
