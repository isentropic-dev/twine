mod graph;

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{visit_mut::VisitMut, Expr, Ident, Member, TypePath};
use twine_core::graph::ComponentGraph;

use crate::utils::IdentExt;

use super::{Components, Extracted};

/// Represents a model of the composed component.
///
/// This struct is responsible for analyzing component dependencies, building a
/// component graph, and generating the final `Twine` computation chain.
pub(crate) struct Model {
    /// The top-level input type for the composed component.
    input: TypePath,

    /// The extracted component definitions and type mappings.
    components: Components,

    /// The component graph representing dependencies.
    graph: ComponentGraph,

    /// The parsed expressions defining how each component receives its input.
    input_exprs: HashMap<Ident, Expr>,
}

impl Model {
    /// Creates a new `Model` from extracted macro input.
    pub(crate) fn new(
        Extracted {
            input,
            components,
            connections,
        }: &Extracted,
    ) -> Self {
        let input_exprs = connections
            .fields
            .iter()
            .cloned()
            .map(|field| {
                let Member::Named(ident) = field.member else {
                    unreachable!("An `ExprStruct` only contains named fields.");
                };

                // Enable LSP support.
                let mut expr = field.expr;
                AdjustSpan.visit_expr_mut(&mut expr);

                (ident, expr)
            })
            .collect();

        Self {
            input: input.clone(),
            components: components.clone(),
            graph: graph::build_graph(&input_exprs),
            input_exprs,
        }
    }

    /// Generates the `Twine` chain for the composed component.
    ///
    /// This function:
    /// - Determines the execution order of components.
    /// - Generates intermediate output types for passing data between components.
    /// - Constructs a `Twine` chain to call components sequentially.
    ///
    /// # Errors
    ///
    /// Returns a `compile_error!` token stream if:
    /// - A cyclic dependency is detected.
    /// - No components are defined.
    pub(crate) fn generate_twine_chain(&self) -> Result<TokenStream, TokenStream> {
        let call_order: Vec<_> = self
            .graph
            .call_order()
            .map_err(|err| {
                quote! {
                    compile_error!(#err)
                }
            })?
            .filter_map(|comp_name| {
                if comp_name == "__input" {
                    None
                } else {
                    Some(format_ident!("{comp_name}"))
                }
            })
            .collect();

        if call_order.is_empty() {
            return Err(quote! {
                compile_error!("At least one component is required.")
            });
        }

        let input = &self.input;
        let template = &self.components.template;

        let output_types = &self.generate_output_types(&call_order);
        let component_calls = self.generate_component_calls(&call_order);
        let last_output_struct = component_output_struct(call_order.last().unwrap());

        Ok(quote! {
            #(#output_types)*

            twine_core::Twine::<#input>::new()
                #(.then(#component_calls))*
                .then_fn(|(input, #last_output_struct { #(#call_order),* })| #template {
                    #(#call_order),*
                })
                .build()
        })
    }

    /// Generates intermediate output types for components.
    ///
    /// These types store the outputs of previously called components, allowing
    /// data to be passed through the `Twine` chain in a structured way.
    fn generate_output_types(&self, call_order: &[Ident]) -> Vec<TokenStream> {
        call_order
            .iter()
            .enumerate()
            .map(|(index, component)| {
                let output_struct = component_output_struct(component);
                let fields = call_order[..=index]
                    .iter()
                    .map(|comp_name| {
                        let types_trait = &self.components.types_trait;
                        let comp_type = comp_name.upper_camel_case();
                        quote! {
                            #comp_name: <<() as #types_trait>::#comp_type as twine_core::Component>::Output
                        }
                    });

                quote! {
                    struct #output_struct {
                        #(#fields),*
                    }
                }
            })
            .collect()
    }

    /// Generates the `.then()` calls for each component in the chain.
    ///
    /// Each `.then()` call:
    /// - Extracts the correct input expression for a component.
    /// - Passes outputs from previous components to the next stage.
    /// - Ensures that each component receives the accumulated outputs of all
    ///   preceding components in the execution order.
    fn generate_component_calls(&self, call_order: &[Ident]) -> Vec<TokenStream> {
        call_order
            .iter()
            .enumerate()
            .map(|(index, component)| {
                let input_type = &self.input;
                let comp_input_expr = self.input_exprs.get(component);
                let comp_output_struct = component_output_struct(component);

                if index == 0 {
                    quote! {
                        components.#component.map(
                            |input: &#input_type| #comp_input_expr,
                            |input, #component| (
                                input,
                                #comp_output_struct {
                                    #component
                                }
                            )
                        )
                    }
                } else {
                    let prev_fields: Vec<_> = call_order[..index].iter().collect();
                    let prev_output_struct = component_output_struct(prev_fields.last().unwrap());

                    quote! {
                        components.#component.map(
                            |(input, output): &(#input_type, #prev_output_struct)| #comp_input_expr,
                            |(input, #prev_output_struct { #(#prev_fields),* }), #component| (
                                input,
                                #comp_output_struct {
                                    #(#prev_fields),*,
                                    #component
                                }
                            )
                        )
                    }
                }
            })
            .collect()
    }
}

/// Generates a struct name for storing intermediate output values.
fn component_output_struct(component: &Ident) -> Ident {
    component.upper_camel_case().with_prefix("__OutputWith")
}

/// A visitor that adjusts spans recursively.
///
/// This is needed to maintain LSP support inside the macro.
struct AdjustSpan;

impl VisitMut for AdjustSpan {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        ident.set_span(Span::call_site());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_quote;

    #[test]
    fn generate_a_simple_twine_chain() {
        let template: TypePath = parse_quote!(TestComponents);
        let extracted = Extracted {
            input: parse_quote!(TestInput),
            components: Components::from(template),
            connections: parse_quote! {
                Connections {
                    first: input.value,
                    second: output.first,
                    third: output.second,
                }
            },
        };

        let model = Model::new(&extracted);

        assert_eq!(
            model
                .generate_twine_chain()
                .expect("Should successfully generate the code.")
                .to_string(),
            (quote! {
                struct __OutputWithFirst {
                    first: <<() as TestComponentsTypes>::First as twine_core::Component>::Output
                }

                struct __OutputWithSecond {
                    first: <<() as TestComponentsTypes>::First as twine_core::Component>::Output,
                    second: <<() as TestComponentsTypes>::Second as twine_core::Component>::Output
                }

                struct __OutputWithThird {
                    first: <<() as TestComponentsTypes>::First as twine_core::Component>::Output,
                    second: <<() as TestComponentsTypes>::Second as twine_core::Component>::Output,
                    third: <<() as TestComponentsTypes>::Third as twine_core::Component>::Output
                }

                twine_core::Twine::<TestInput>::new()
                    .then(components.first.map(
                        |input: &TestInput| input.value,
                        |input, first| (
                            input,
                            __OutputWithFirst {
                                first
                            }
                        )
                    ))
                    .then(components.second.map(
                        |(input, output): &(TestInput, __OutputWithFirst)| output.first,
                        |(input, __OutputWithFirst { first }), second| (
                            input,
                            __OutputWithSecond {
                                first,
                                second
                            }
                        )
                    ))
                    .then(components.third.map(
                        |(input, output): &(TestInput, __OutputWithSecond)| output.second,
                        |(input, __OutputWithSecond { first, second }), third| (
                            input,
                            __OutputWithThird {
                                first,
                                second,
                                third
                            }
                        )
                    ))
                    .then_fn(|(input, __OutputWithThird { first, second, third })| TestComponents {
                        first,
                        second,
                        third
                    })
                    .build()
            })
            .to_string(),
        );
    }
}
