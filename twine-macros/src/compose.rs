#![allow(dead_code)] // Just for now...

mod generate;
mod graph;
mod parse;

use petgraph::{
    algo::toposort,
    graph::{DiGraph, NodeIndex},
    Direction,
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ExprStruct, Ident, Path};

pub(crate) fn expand(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(item as ComponentDefinition);
    let graph = definition.into();
    generate::code(&graph).into()
}

/// Defines a composed component.
struct ComponentDefinition {
    component_type: Path,
    input_type: Path,
    components: Vec<ComponentInstance>,
}

/// Represents an instance of an inner component.
struct ComponentInstance {
    name: Ident,
    component_type: Path,
    input_struct: ExprStruct,
}

struct ComponentGraph {
    definition: ComponentDefinition,
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

    /// Returns the components in the proper call order.
    pub fn call_order(&self) -> Vec<usize> {
        toposort(&self.dependencies, None)
            // https://github.com/isentropic-dev/twine/issues/29
            .expect("Cycle detected in component dependencies")
            .into_iter()
            .map(|node_index| self.dependencies[node_index])
            .collect()
    }
}
