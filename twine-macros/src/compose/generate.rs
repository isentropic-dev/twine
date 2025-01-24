use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;

use super::{ComponentDefinition, ComponentGraph, InputField, InputSchema};

/// Expands a `ComponentGraph` into a Rust module.
///
/// This function generates a module containing:
/// - `Config`: Stores configuration values for component instances.
/// - `Input`: Defines the structured input schema.
/// - `Output`: Represents the output schema for component instances.
/// - `check_types`: A function that provides compile-time type validation.
pub(crate) fn expand(graph: &ComponentGraph) -> TokenStream {
    let name = &graph.definition.name;
    let config = generate_config(&graph.definition);
    let input = generate_input(&graph.definition);
    let output = generate_output(&graph.definition);
    let type_check_fn = generate_type_check_fn(graph);

    quote! {
        mod #name {
            use super::*;
            #config
            #input
            #output
            #type_check_fn
        }
    }
}

/// Generates a `Config` struct with configuration fields for each component instance.
fn generate_config(definition: &ComponentDefinition) -> TokenStream {
    let fields = definition.components.iter().map(|instance| {
        let name = &instance.name;
        let module = &instance.module;
        quote! { pub #name: #module::Config, }
    });

    quote! {
        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
        pub struct Config {
            #(#fields)*
        }
    }
}

/// Generates an `Input` struct and nested modules for hierarchical input fields.
fn generate_input(definition: &ComponentDefinition) -> TokenStream {
    let fields = generate_input_fields(&definition.input_schema);
    let nested_modules = generate_nested_modules(&definition.input_schema);

    quote! {
        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
        pub struct Input {
            #(#fields)*
        }

        #(#nested_modules)*
    }
}

/// Generates fields for the `Input` struct, supporting both simple and nested types.
fn generate_input_fields(input_schema: &InputSchema) -> Vec<TokenStream> {
    input_schema
        .iter()
        .sorted_by_key(|(ident, _)| ident.to_string())
        .map(|(field_name, field_value)| match field_value {
            InputField::Type(field_type) => quote! { pub #field_name: #field_type, },
            InputField::Struct(_) => quote! { pub #field_name: #field_name::Input, },
        })
        .collect()
}

/// Recursively generates nested Rust modules for structured input fields.
fn generate_nested_modules(input_schema: &InputSchema) -> Vec<TokenStream> {
    input_schema
        .iter()
        .sorted_by_key(|(ident, _)| ident.to_string())
        .filter_map(|(mod_name, field_value)| {
            if let InputField::Struct(nested_schema) = field_value {
                let nested_fields = generate_input_fields(nested_schema);
                let nested_modules = generate_nested_modules(nested_schema);

                Some(quote! {
                    pub mod #mod_name {
                        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
                        pub struct Input {
                            #(#nested_fields)*
                        }

                        #(#nested_modules)*
                    }
                })
            } else {
                None
            }
        })
        .collect()
}

/// Generates an `Output` struct with output fields for each component instance.
fn generate_output(definition: &ComponentDefinition) -> TokenStream {
    let fields = definition.components.iter().map(|instance| {
        let name = &instance.name;
        let module = &instance.module;
        quote! { pub #name: #module::Output, }
    });

    quote! {
        #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
        pub struct Output {
            #(#fields)*
        }
    }
}

/// Generates a function for compile-time input validation.
fn generate_type_check_fn(graph: &ComponentGraph) -> TokenStream {
    let input_fields = graph
        .definition
        .input_schema
        .keys()
        .sorted()
        .map(|field_name| quote! { #field_name });

    // Bring inputs into scope.
    let inputs = quote! {
        let Input { #(#input_fields),* } = Input::default();
    };

    // Bring required outputs into scope.
    let component_outputs: Vec<_> = graph
        .definition
        .components
        .iter()
        .enumerate()
        .filter(|(index, _instance)| graph.is_used_as_input(*index))
        .map(|(_index, instance)| {
            let name = &instance.name;
            let mod_input = &instance.module;
            quote! { let #name = #mod_input::Output::default(); }
        })
        .collect();

    // Check each component's input expression.
    let component_inputs: Vec<_> = graph
        .definition
        .components
        .iter()
        .map(|instance| {
            let input_struct = &instance.input_struct;
            quote! { let _ = #input_struct; }
        })
        .collect();

    quote! {
        fn check_types() {
            #inputs
            #(#component_outputs)*
            #(#component_inputs)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::{parse2, parse_quote, File};

    /// Compares two `TokenStream`s for easier debugging of macro output differences.
    fn assert_eq_pretty(expected: &TokenStream, actual: &TokenStream) {
        fn pretty(ts: &TokenStream) -> String {
            let syntax_tree = parse2::<File>(ts.clone()).expect("Failed to parse TokenStream");
            prettyplease::unparse(&syntax_tree)
        }
        let expected = pretty(expected);
        let actual = pretty(actual);
        assert!(
            expected == actual,
            "\n--- Expected ---\n{expected}\n\n--- Actual ---\n{actual}"
        );
    }

    #[test]
    fn generate_config_works() {
        let generated = generate_config(
            &(parse_quote! {
                test {
                    first => example {
                        x: 1.0
                    }

                    second => example {
                        x: 2.0
                    }

                    third => another {
                        y: 3.0
                    }
                }
            }),
        );
        let expected = quote! {
            #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
            pub struct Config {
                pub first: example::Config,
                pub second: example::Config,
                pub third: another::Config,
            }
        };
        assert_eq_pretty(&expected, &generated);
    }

    #[test]
    fn generate_input_works() {
        let generated = generate_input(
            &(parse_quote! {
                test {
                    Input {
                        time: f64,
                        hour: usize,
                        indoor: {
                            occupancy: u32,
                            pressure: f64,
                            temp_setpoint: f64,
                        },
                        thermostat_control: {
                            auto_mode: bool,
                        }
                    }
                }
            }),
        );

        let expected = quote! {
            #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
            pub struct Input {
                pub hour: usize,
                pub indoor: indoor::Input,
                pub thermostat_control: thermostat_control::Input,
                pub time: f64,
            }

            pub mod indoor {
                #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
                pub struct Input {
                    pub occupancy: u32,
                    pub pressure: f64,
                    pub temp_setpoint: f64,
                }
            }

            pub mod thermostat_control {
                #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
                pub struct Input {
                    pub auto_mode: bool,
                }
            }
        };

        assert_eq_pretty(&expected, &generated);
    }

    #[test]
    fn generate_output_works() {
        let generated = generate_output(
            &(parse_quote! {
                test {
                    first_one => first {
                        x: 1.0
                    }

                    second_one => second {
                        x: first_one.y
                    }
                }
            }),
        );
        let expected = quote! {
            #[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
            pub struct Output {
                pub first_one: first::Output,
                pub second_one: second::Output,
            }
        };
        assert_eq_pretty(&expected, &generated);
    }

    #[test]
    fn generate_type_check_fn_works() {
        let definition: ComponentDefinition = parse_quote! {
            test {
                Input {
                    x: bool,
                    y: i32,
                    z: f64,
                    extra: {
                        verbose: bool,
                    },
                }

                first_one => first {
                    x,
                }

                second_one => second {
                    y: first_one.y,
                    z: extra.verbose,
                }
            }
        };
        let graph = definition.into();
        let generated = generate_type_check_fn(&graph);
        let expected = quote! {
            fn check_types() {
                let Input { extra, x, y, z } = Input::default();
                let first_one = first::Output::default();
                let _ = first::Input { x };
                let _ = second::Input {
                    y: first_one.y,
                    z: extra.verbose,
                };
            }
        };
        assert_eq_pretty(&expected, &generated);
    }
}
