mod generate;
mod graph;
mod parse;

use std::collections::HashMap;

use petgraph::{
    graph::{DiGraph, NodeIndex},
    Direction,
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ExprStruct, Ident, Path, Type};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(input as ComponentDefinition);
    let graph = definition.into();
    generate::expand(&graph).into()
}

/// Defines a composed component.
#[derive(Debug)]
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
#[derive(Debug)]
enum InputField {
    Type(Type),
    Struct(InputSchema),
}

/// Represents an instance of an inner component.
#[derive(Debug)]
struct ComponentInstance {
    name: Ident,
    module: Path,
    input_struct: ExprStruct,
}

impl ComponentGraph {
    /// Checks if a component's output is used as an input elsewhere.
    pub fn is_used_as_input(&self, component_index: usize) -> bool {
        let node_index = NodeIndex::new(component_index);

        // Check for any outgoing edges.
        self.dependencies
            .neighbors_directed(node_index, Direction::Outgoing)
            .next()
            .is_some()
    }
}
