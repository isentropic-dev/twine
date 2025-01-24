use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{ComponentDefinition, ComponentGraph, InputField, InputSchema};

/// Generates a Rust module from a `ComponentGraph`.
///
/// - Defines `Config`, `Input`, and `Output` structs.
/// - Includes `check_types` to validate input types at compile time.
/// - Provides `create`, which returns a callable component instance.
///
/// This function produces a `mod` block containing the generated structs
/// and functions necessary for the composed component.
pub(crate) fn generate_module(graph: &ComponentGraph) -> TokenStream {
    let definition = &graph.definition;
    let name = &definition.name;
    let config = generate_config(definition);
    let input = generate_input(definition);
    let output = generate_output(definition);
    let type_check_fn = generate_check_types_fn(graph);
    let create_fn = generate_create_fn(graph);

    quote! {
        mod #name {
            use super::*;
            #config
            #input
            #output
            #type_check_fn
            #create_fn
        }
    }
}

/// Generates a `Config` struct holding each component’s configuration.
fn generate_config(definition: &ComponentDefinition) -> TokenStream {
    let fields = definition.components.iter().map(|instance| {
        let name = &instance.name;
        let module = &instance.module;
        quote! { pub #name: #module::Config, }
    });

    quote! {
        #[derive(Debug, Default)]
        #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
        pub struct Config {
            #(#fields)*
        }
    }
}

/// Generates an `Input` struct with nested modules for hierarchical fields.
fn generate_input(definition: &ComponentDefinition) -> TokenStream {
    let fields = create_input_fields(&definition.input_schema);
    let nested_modules = create_nested_module(&definition.input_schema);

    quote! {
        #[derive(Debug, Default)]
        #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
        pub struct Input {
            #(#fields)*
        }

        #(#nested_modules)*
    }
}

/// Creates fields for the `Input` struct based on the schema.
///
/// Fields are sorted for consistency.
fn create_input_fields(input_schema: &InputSchema) -> Vec<TokenStream> {
    input_schema
        .iter()
        .sorted_by_key(|(ident, _)| ident.to_string())
        .map(|(field_name, field_value)| match field_value {
            InputField::Type(field_type) => quote! { pub #field_name: #field_type, },
            InputField::Struct(_) => quote! { pub #field_name: #field_name::Input, },
        })
        .collect()
}

/// Recursively creates nested modules for hierarchical `Input` fields.
///
/// Each module includes an `Input` struct.
fn create_nested_module(input_schema: &InputSchema) -> Vec<TokenStream> {
    input_schema
        .iter()
        .sorted_by_key(|(ident, _)| ident.to_string())
        .filter_map(|(mod_name, field_value)| {
            if let InputField::Struct(nested_schema) = field_value {
                let nested_fields = create_input_fields(nested_schema);
                let nested_modules = create_nested_module(nested_schema);

                Some(quote! {
                    pub mod #mod_name {
                        #[derive(Debug, Default)]
                        #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
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

/// Generates an `Output` struct collecting component outputs.
fn generate_output(definition: &ComponentDefinition) -> TokenStream {
    let fields = definition.components.iter().map(|instance| {
        let name = &instance.name;
        let module = &instance.module;
        quote! { pub #name: #module::Output, }
    });

    quote! {
        #[derive(Debug, Default)]
        #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
        pub struct Output {
            #(#fields)*
        }
    }
}

/// Generates `check_types` to ensure components receive correctly typed inputs at compile time.
fn generate_check_types_fn(graph: &ComponentGraph) -> TokenStream {
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

    // Initialize each component.
    let initialize_components: Vec<_> = call_order
        .iter()
        .map(|&index| {
            let component = &definition.components[index];
            let name = &component.name;
            let module = &component.module;
            let name_fn = format_ident!("{}_fn", name);
            quote! {
                let #name_fn = #module::create(config.#name);
            }
        })
        .collect();

    // Gather all input fields.
    let input_fields: Vec<_> = definition
        .input_schema
        .keys()
        .sorted_by_key(ToString::to_string)
        .map(|field_name| quote! { #field_name })
        .collect();

    // Call each component.
    let call_components: Vec<_> = call_order
        .iter()
        .map(|&index| {
            let component = &definition.components[index];
            let name = &component.name;
            let name_fn = format_ident!("{}_fn", name);
            let input_struct = &component.input_struct;
            quote! {
                let #name = #name_fn(#input_struct);
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
        pub fn create(config: Config) -> impl Fn(Input) -> Output {
            #(#initialize_components)*
            move |Input { #(#input_fields),* }| {
                #(#call_components)*
                Output {
                    #(#output_fields)*
                }
            }
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
            #[derive(Debug, Default)]
            #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
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
            #[derive(Debug, Default)]
            #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
            pub struct Input {
                pub hour: usize,
                pub indoor: indoor::Input,
                pub thermostat_control: thermostat_control::Input,
                pub time: f64,
            }

            pub mod indoor {
                #[derive(Debug, Default)]
                #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
                pub struct Input {
                    pub occupancy: u32,
                    pub pressure: f64,
                    pub temp_setpoint: f64,
                }
            }

            pub mod thermostat_control {
                #[derive(Debug, Default)]
                #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
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
            #[derive(Debug, Default)]
            #[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
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
        let generated = generate_check_types_fn(&graph);
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

    #[test]
    fn generate_create_fn_works() {
        let definition: ComponentDefinition = parse_quote! {
            test {
                finalizer => subtractor {
                    value_in: multiplier.result,
                    value_out: offset,
                }

                adder_b => adder {
                    value_in: adder_a.value_out,
                }

                Input {
                    input_value: f64,
                    factor: f64,
                    offset: f64,
                }

                adder_a => adder {
                    value_in: input_value,
                }


                multiplier => multiplier {
                    value_in: adder_b.value_out,
                    factor,
                }

            }
        };
        let graph = definition.into();
        let generated = generate_create_fn(&graph);
        let expected = quote! {
            pub fn create(config: Config) -> impl Fn(Input) -> Output {
                let adder_a_fn = adder::create(config.adder_a);
                let adder_b_fn = adder::create(config.adder_b);
                let multiplier_fn = multiplier::create(config.multiplier);
                let finalizer_fn = subtractor::create(config.finalizer);

                move |Input { factor, input_value, offset }| {
                    let adder_a = adder_a_fn(adder::Input {
                        value_in: input_value,
                    });

                    let adder_b = adder_b_fn(adder::Input {
                        value_in: adder_a.value_out,
                    });

                    let multiplier = multiplier_fn(multiplier::Input {
                        value_in: adder_b.value_out,
                        factor,
                    });

                    let finalizer = finalizer_fn(subtractor::Input {
                        value_in: multiplier.result,
                        value_out: offset,
                    });

                    Output {
                        adder_a,
                        adder_b,
                        multiplier,
                        finalizer,
                    }
                }
            }
        };
        assert_eq_pretty(&expected, &generated);
    }
}
