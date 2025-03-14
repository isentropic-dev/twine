use std::collections::HashMap;

use quote::ToTokens;
use syn::{visit::Visit, Expr, ExprField, Ident, Member};
use twine_core::graph::{ComponentGraph, Target};

/// Constructs a `ComponentGraph` by analyzing component input expressions.
///
/// Each component's input expressions are traversed to determine dependencies
/// on the top-level `input` and outputs from other components.
pub(crate) fn build_graph(input_exprs: &HashMap<Ident, Expr>) -> ComponentGraph {
    let mut graph_builder = GraphBuilder::new();

    for (component, expr) in input_exprs {
        match expr {
            Expr::Struct(inputs) => {
                // An `ExprStruct` means the component's input has fields.
                inputs.fields.iter().for_each(|field| {
                    let input_field = match &field.member {
                        Member::Named(ident) => ident.to_string(),
                        Member::Unnamed(index) => index.index.to_string(),
                    };
                    graph_builder
                        .with_target(component, &input_field)
                        .visit_expr(&field.expr);
                });
            }
            expr => {
                // Any other `Expr` means the component's input is set directly.
                graph_builder.with_target(component, "").visit_expr(expr);
            }
        }
    }

    graph_builder.graph
}

/// Helper for building component dependency graphs by traversing expressions.
struct GraphBuilder {
    graph: ComponentGraph,
    current_target: Target,
}

impl GraphBuilder {
    fn new() -> Self {
        Self {
            graph: ComponentGraph::new(),
            current_target: Target {
                component: String::new(),
                input: String::new(),
            },
        }
    }

    fn with_target<T, U>(&mut self, component: &T, input: &U) -> &mut Self
    where
        T: ToString + ?Sized,
        U: ToString + ?Sized,
    {
        self.current_target = Target::new(component.to_string(), input.to_string());
        self
    }
}

impl Visit<'_> for GraphBuilder {
    fn visit_expr_field(&mut self, node: &ExprField) {
        let full_path = node.to_token_stream().to_string().replace(" . ", ".");

        // References the top-level input (`input`) as a special source component.
        if let Some(stripped) = full_path.strip_prefix("input") {
            self.graph.connect(
                match stripped.split_once('.') {
                    // The input type has fields.
                    Some((_, field)) => ("__input", field),

                    // The input type is used directly.
                    None => ("__input", ""),
                },
                self.current_target.clone(),
            );
        }

        // References an output from another component.
        if let Some(stripped) = full_path.strip_prefix("output.") {
            self.graph.connect(
                match stripped.split_once('.') {
                    // The component has an output field.
                    Some((component, output_field)) => (component, output_field),

                    // The component is used directly.
                    None => (stripped, ""),
                },
                self.current_target.clone(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;
    use syn::parse_quote;
    use twine_core::graph::Source;

    fn sorted_incoming(graph: &ComponentGraph, component: &str) -> Vec<Source> {
        graph
            .incoming_connections(component)
            .sorted_by_key(|source| format!("{source:?}"))
            .collect_vec()
    }

    fn sorted_outgoing(graph: &ComponentGraph, component: &str) -> Vec<Target> {
        graph
            .outgoing_connections(component)
            .sorted_by_key(|target| format!("{target:?}"))
            .collect_vec()
    }

    #[test]
    fn build_component_graph_works() {
        let input_exprs: HashMap<Ident, Expr> = vec![
            (
                parse_quote!(first),
                parse_quote!(FirstInput {
                    x: input.value,
                    y: output.second
                }),
            ),
            (parse_quote!(second), parse_quote!(output.first.value)),
            (
                parse_quote!(third),
                parse_quote!(ThirdInput {
                    input_1: output.first.nested.output,
                    input_2: output.second,
                }),
            ),
        ]
        .into_iter()
        .collect();

        let graph = build_graph(&input_exprs);

        // Check incomining connections.
        assert_eq!(
            sorted_incoming(&graph, "first"),
            vec![Source::new("__input", "value"), Source::new("second", "")],
        );
        assert_eq!(
            sorted_incoming(&graph, "second"),
            vec![Source::new("first", "value")],
        );
        assert_eq!(
            sorted_incoming(&graph, "third"),
            vec![
                Source::new("first", "nested.output"),
                Source::new("second", "")
            ],
        );

        // Check outgoing connections.
        assert_eq!(
            sorted_outgoing(&graph, "first"),
            vec![Target::new("second", ""), Target::new("third", "input_1")],
        );
        assert_eq!(
            sorted_outgoing(&graph, "second"),
            vec![Target::new("first", "y"), Target::new("third", "input_2")],
        );
        assert_eq!(sorted_outgoing(&graph, "third"), vec![]);
    }
}
