#![allow(dead_code)] // Just for now...

mod generate;
mod graph;
mod parse;

use petgraph::{algo::toposort, graph::DiGraph};
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
    /// Iterates over components in the proper call order.
    fn iter_components(&self) -> impl Iterator<Item = &ComponentInstance> {
        toposort(&self.dependencies, None)
            // https://github.com/isentropic-dev/twine/issues/29
            .expect("Cycle detected in component dependencies")
            .into_iter()
            .filter_map(|node_index| self.dependencies.node_weight(node_index))
            .map(|&index| &self.definition.components[index])
    }
}
