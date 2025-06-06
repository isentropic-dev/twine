use std::collections::HashMap;

use petgraph::{
    Direction,
    algo::toposort,
    graph::{DiGraph, NodeIndex},
};

/// A directed graph representing connections between components.
#[derive(Debug)]
pub struct ComponentGraph {
    graph: DiGraph<String, Connection>,
    node_map: HashMap<String, NodeIndex>,
}

impl ComponentGraph {
    /// Creates an empty component graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Connects two components with a directed edge.
    ///
    /// This method establishes a connection from a source component's output to
    /// a target component's input. If the components do not exist in the graph,
    /// they are added automatically.
    ///
    /// # Argument Types
    ///
    /// This function accepts any types that implement `Into<Source>` and
    /// `Into<Target>`, respectively. To simplify usage, `Source` and `Target`
    /// implement `From<(T, T)>` where `T: Into<String>`, allowing tuples of
    /// string-like values (e.g., `(&str, &str)`) to be used directly.
    ///
    /// # Arguments
    ///
    /// * `source` - The source component and its output (e.g., `("component_name", "output_name")`).
    /// * `target` - The target component and its input (e.g., `("component_name", "input_name")`).
    ///
    /// # Examples
    ///
    /// ```
    /// use twine_core::graph::ComponentGraph;
    ///
    /// let mut graph = ComponentGraph::new();
    ///
    /// // This establishes the following dependencies:
    /// // - "comp_b.in" depends on "comp_a.out_1"
    /// // - "comp_c.in_1" depends on "comp_a.out_2"
    /// // - "comp_c.in_2" depends on "comp_b.out"
    /// graph.connect(("comp_a", "out_1"), ("comp_b", "in"));
    /// graph.connect(("comp_a", "out_2"), ("comp_c", "in_1"));
    /// graph.connect(("comp_b", "out"), ("comp_c", "in_2"));
    /// ```
    pub fn connect<S: Into<Source>, T: Into<Target>>(&mut self, source: S, target: T) {
        let source = source.into();
        let target = target.into();

        let source_index = self.get_or_add_component(&source.component);
        let target_index = self.get_or_add_component(&target.component);

        self.graph
            .add_edge(source_index, target_index, Connection { source, target });
    }

    /// Returns an iterator over components in topological order.
    ///
    /// This ensures that each component appears only after all its dependencies.
    ///
    /// # Returns
    ///
    /// An iterator yielding component names (`&str`) in execution order.
    ///
    /// # Errors
    ///
    /// Returns an error if the graph contains a cycle.
    ///
    /// # Examples
    ///
    /// ```
    /// use twine_core::graph::ComponentGraph;
    ///
    /// let mut graph = ComponentGraph::new();
    /// graph.connect(("comp_a", "out"), ("comp_b", "in"));
    /// graph.connect(("comp_b", "out"), ("comp_c", "in"));
    ///
    /// let order: Vec<_> = graph.call_order().unwrap().collect();
    /// assert_eq!(order, vec!["comp_a", "comp_b", "comp_c"]);
    /// ```
    pub fn call_order(&self) -> Result<impl Iterator<Item = &str>, String> {
        Ok(toposort(&self.graph, None)
            .map_err(|_| "Cycle detected in component dependencies")?
            .into_iter()
            .map(|node_index| self.graph[node_index].as_str()))
    }

    /// Returns the number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of edges in the graph.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns an iterator over all incoming connections for a given component.
    ///
    /// # Arguments
    ///
    /// * `component` - The name of the component to query.
    ///
    /// # Returns
    ///
    /// An iterator over `Source` objects representing incoming connections.
    pub fn incoming_connections(&self, component: &str) -> impl Iterator<Item = Source> {
        self.node_map
            .get(component)
            .into_iter()
            .flat_map(|&node_index| self.graph.edges_directed(node_index, Direction::Incoming))
            .map(|edge| edge.weight().source.clone())
    }

    /// Returns an iterator over all outgoing connections for a given component.
    ///
    /// # Arguments
    ///
    /// * `component` - The name of the component to query.
    ///
    /// # Returns
    ///
    /// An iterator over `Target` objects representing outgoing connections.
    pub fn outgoing_connections(&self, component: &str) -> impl Iterator<Item = Target> {
        self.node_map
            .get(component)
            .into_iter()
            .flat_map(|&node_index| self.graph.edges_directed(node_index, Direction::Outgoing))
            .map(|edge| edge.weight().target.clone())
    }

    /// Returns the node index for a component, adding it to the graph if it does not exist.
    fn get_or_add_component<T: Into<String>>(&mut self, component: T) -> NodeIndex {
        let component = component.into();
        *self
            .node_map
            .entry(component.clone())
            .or_insert_with(|| self.graph.add_node(component))
    }
}

/// Represents the source of a connection in the graph.
///
/// A source consists of a `component` name and an `output` port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub component: String,
    pub output: String,
}

impl Source {
    #[must_use]
    pub fn new(component: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            output: output.into(),
        }
    }
}

/// Represents the target of a connection in the graph.
///
/// A target consists of a `component` name and an `input` port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub component: String,
    pub input: String,
}

impl Target {
    #[must_use]
    pub fn new(component: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            input: input.into(),
        }
    }
}

/// Represents a directed connection between two components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub source: Source,
    pub target: Target,
}

impl<T: Into<String>> From<(T, T)> for Source {
    fn from((component, output): (T, T)) -> Self {
        Self {
            component: component.into(),
            output: output.into(),
        }
    }
}

impl<T: Into<String>> From<(T, T)> for Target {
    fn from((component, input): (T, T)) -> Self {
        Self {
            component: component.into(),
            input: input.into(),
        }
    }
}

impl Default for ComponentGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_connections() {
        let mut graph = ComponentGraph::new();

        graph.connect(("comp_a", "out_1"), ("comp_b", "in"));
        graph.connect(("comp_a", "out_2"), ("comp_c", "in_1"));
        graph.connect(("comp_a", "out_3"), ("comp_c", "in_2"));
        graph.connect(("comp_b", "out"), ("comp_c", "in_3"));

        assert_eq!(graph.graph.node_count(), 3);
        assert_eq!(graph.graph.edge_count(), 4);

        let index_a = graph.node_map["comp_a"];
        let index_b = graph.node_map["comp_b"];
        let index_c = graph.node_map["comp_c"];

        assert!(graph.graph.contains_edge(index_a, index_b));
        assert!(graph.graph.contains_edge(index_a, index_c));
        assert!(graph.graph.contains_edge(index_b, index_c));
    }

    #[test]
    fn call_order_iterator() {
        let mut graph = ComponentGraph::new();

        graph.connect(("comp_a", "out"), ("comp_b", "in"));
        graph.connect(("comp_b", "out"), ("comp_c", "in"));
        graph.connect(("comp_c", "out"), ("comp_d", "in"));
        graph.connect(("comp_d", "out"), ("comp_e", "in"));

        let call_order: Vec<_> = graph.call_order().unwrap().collect();

        assert_eq!(
            call_order,
            vec!["comp_a", "comp_b", "comp_c", "comp_d", "comp_e"]
        );
    }

    #[test]
    fn connections_iterators() {
        let mut graph = ComponentGraph::new();

        graph.connect(("A", "out"), ("B", "in"));
        graph.connect(("B", "out"), ("C", "in"));
        graph.connect(("C", "out"), ("D", "in"));
        graph.connect(("A", "out"), ("D", "in"));

        // Check incoming connections.
        assert_eq!(
            graph.incoming_connections("B").collect::<Vec<_>>(),
            vec![("A", "out").into()],
        );

        assert_eq!(
            graph.incoming_connections("C").collect::<Vec<_>>(),
            vec![("B", "out").into()]
        );

        assert_eq!(
            graph.incoming_connections("D").collect::<Vec<_>>(),
            vec![("A", "out").into(), ("C", "out").into()]
        );

        // Check outgoing connections.
        assert_eq!(
            graph.outgoing_connections("A").collect::<Vec<_>>(),
            vec![("D", "in").into(), ("B", "in").into()]
        );

        assert_eq!(
            graph.outgoing_connections("B").collect::<Vec<_>>(),
            vec![("C", "in").into()]
        );

        assert_eq!(
            graph.outgoing_connections("C").collect::<Vec<_>>(),
            vec![("D", "in").into()]
        );

        // No incoming for "A" and no outgoing for "D".
        assert!(graph.incoming_connections("A").next().is_none());
        assert!(graph.outgoing_connections("D").next().is_none());
    }
}
