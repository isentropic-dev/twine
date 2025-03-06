mod extract;
mod model;
mod parse;

use proc_macro2::TokenStream;
use quote::quote;

use extract::{Components, Extracted};
use model::Model;

pub(crate) use parse::{Parsed, ParsedAttr, ParsedItem};

impl Parsed {
    /// Expands parsed input into the final generated code.
    ///
    /// On error, returns generated code containing compile-time diagnostics to
    /// provide LSP support and assist the user in correcting mistakes.
    pub(crate) fn try_expand(self) -> Result<TokenStream, TokenStream> {
        let Self {
            name,
            attrs,
            vis,
            stmts,
        } = self;

        // Extracts the input type, components, and their connections.
        let extracted: Extracted = stmts.as_slice().try_into()?;

        // Generates the `twine_core::Twine` chain based on the connections.
        let twine_chain = Model::new(&extracted).generate_twine_chain()?;

        let Extracted {
            input,
            components,
            connections,
        } = extracted;

        let connections_type = generate_connections_type(&components);

        let Components {
            concrete,
            inputs,
            outputs,
            ..
        } = components;

        let new_fn_doc = format!("Creates a new `{name}` from instantiated components.");

        Ok(quote! {
            #(#attrs)*
            #vis struct #name {
                component: Box<
                    dyn twine_core::Component<
                        Input = #input,
                        Output = #outputs,
                        Error = twine_core::TwineError,
                    >,
                >,
            }

            impl #name {
                /// Provides LSP support by validating the types of connection expressions.
                #[allow(clippy::no_effect)]
                fn __lsp_support(input: #input, output: #outputs) {
                    #connections_type
                    let _: #inputs = #connections;
                }

                #[doc = #new_fn_doc]
                #vis fn new(components: #concrete) -> Self {
                    use twine_core::Component;

                    let component = Box::new({
                        #twine_chain
                    });

                    Self { component }
                }
            }

            impl twine_core::Component for #name {
                type Input = #input;
                type Output = #outputs;
                type Error = twine_core::TwineError;

                fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
                    self.component.call(input)
                }
            }
        })
    }
}

/// Generates the `Connections` type alias used to define component input wiring.
fn generate_connections_type(components: &Components) -> TokenStream {
    let comp_inputs = &components.inputs;
    let template_name = components.template_name();

    let docstring = format!(
        r"Defines input connections for components within `{template_name}`.

Each field of this struct corresponds to a component in `{template_name}`. The field's
value specifies how that component receives its input, defined using expressions.

# Usage

- Reference the top-level input using `input`.
- Reference outputs from other components using `output.component_name`."
    );

    quote! {
        #[doc = #docstring]
        type Connections = #comp_inputs;
    }
}
