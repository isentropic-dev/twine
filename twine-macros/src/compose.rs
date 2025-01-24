mod generate;
mod graph;
mod parse;

use std::collections::HashMap;

use petgraph::graph::DiGraph;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ExprStruct, Ident, Path, Type};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(input as ComponentDefinition);
    let graph = definition.into();
    generate::expand(&graph).into()
}

/// Defines a composed component.
struct ComponentDefinition {
    /// Component name.
    name: Ident,

    /// Schema defining the `Input` type.
    input_schema: InputSchema,

    /// Inner component instances.
    components: Vec<ComponentInstance>,
}

struct ComponentGraph {
    definition: ComponentDefinition,
    #[allow(dead_code)] // Just for now...
    dependencies: DiGraph<usize, graph::Connection>,
}

impl From<ComponentDefinition> for ComponentGraph {
    fn from(definition: ComponentDefinition) -> Self {
        let dependencies = graph::build_graph(&definition.components);
        Self {
            definition,
            dependencies,
        }
    }
}

/// Maps field identifiers to their corresponding input types or structures.
type InputSchema = HashMap<Ident, InputField>;

/// Represents an input field as a type or nested schema.
enum InputField {
    Type(Type),
    Struct(InputSchema),
}

/// Represents an instance of an inner component.
struct ComponentInstance {
    name: Ident,
    module: Path,
    input_struct: ExprStruct,
}

impl ComponentDefinition {
    /// Checks if a component's output is used as an input elsewhere.
    ///
    /// This is a temporary placeholder until a full dependency graph is in
    /// place. Once a graph-based approach is available, this method will be
    /// replaced by something that uses it.
    ///
    /// # Parameters
    /// - `component_name`: The name of the component to check.
    ///
    /// # Returns
    /// - `true` if `component_name` appears in any input field.
    /// - `false` otherwise.
    fn is_used_as_input(&self, component_name: &Ident) -> bool {
        let component_str = format!("{component_name}.");
        self.components.iter().any(|instance| {
            instance.input_struct.fields.iter().any(|field| {
                let expr_str = quote::ToTokens::to_token_stream(&field.expr)
                    .to_string()
                    .replace(' ', "");
                expr_str.contains(&component_str)
            })
        })
    }
}
