use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};
use quote::ToTokens;
use syn::{Expr, ExprField, ExprPath, Member};

use super::ComponentInstance;

/// Builds a directed dependency graph from component instances.
///
/// Each component becomes a node in the graph, indexed by its position in the
/// `components` list. Directed edges represent dependencies inferred from input
/// field references.
///
/// How It Works:
///
/// - Each component's input struct defines its field assignments.
/// - If an input field references another component's output, a directed edge
///   is added from the source component to the dependent (target) component.
///   For example, `weather.temperature` used by `first_house` results in an
///   edge from `weather` to `first_house`.
/// - Nested field accesses, such as `component.with.nested.output`, are stored
///   as `"with.nested.output"` to preserve full paths.
/// - References that resemble component names but refer to external inputs
///   are ignored.
///
/// # Returns
///
/// A `DiGraph<usize, Connection>` where:
/// - Nodes are component indices.
/// - Edges represent dependencies, labeled with `Connection`.
pub(crate) fn build_graph(components: &[ComponentInstance]) -> DiGraph<usize, Connection> {
    let mut graph = DiGraph::new();

    let node_map: HashMap<String, NodeIndex> = components
        .iter()
        .enumerate()
        .map(|(index, component)| (component.name.to_string(), graph.add_node(index)))
        .collect();

    for (target_index, target_component) in components.iter().enumerate() {
        for (source_component, connection) in find_incoming_connections(target_component) {
            if let Some(&source_index) = node_map.get(&source_component) {
                graph.add_edge(source_index, NodeIndex::new(target_index), connection);
            }
        }
    }

    graph
}

/// Represents a directed connection between two components.
///
/// This struct defines an input-output field pair, tracking dependencies
/// between components. Each connection represents data flow from a source
/// component’s output field to a target component’s input field.
///
/// It is primarily used for dependency resolution and graph visualization.
#[derive(Debug, PartialEq)]
pub(crate) struct Connection {
    /// Name of the output field on the source component.
    source: String,
    /// Name of the input field on the target component.
    target: String,
}

/// Finds dependencies by analyzing input field expressions.
///
/// Extracts references to outputs from other components.
///
/// # Returns
///
/// A `Vec<(String, Connection)>`, where:
/// - `String` is the source component providing an output.
/// - `Connection` maps the source component's output to the current component's input.
fn find_incoming_connections(component: &ComponentInstance) -> Vec<(String, Connection)> {
    iter_named_fields(component)
        .flat_map(|(field_name, field_expr)| {
            get_component_outputs(field_expr).into_iter().map(
                move |(component_name, output_name)| {
                    (
                        component_name,
                        Connection {
                            source: output_name,
                            target: field_name.clone(),
                        },
                    )
                },
            )
        })
        .collect()
}

/// Iterates over a component's named fields, ignoring unnamed (tuple) fields.
///
/// # Returns
///
/// An iterator yielding `(field_name, field_expr)` pairs, where:
/// - `field_name` is the field’s name as a `String`.
/// - `field_expr` is a reference to the field's expression (`&Expr`).
fn iter_named_fields(component: &ComponentInstance) -> impl Iterator<Item = (String, &Expr)> {
    component.input_struct.fields.iter().filter_map(|field| {
        if let Member::Named(ident) = &field.member {
            Some((ident.to_string(), &field.expr))
        } else {
            None
        }
    })
}

/// Extracts all `component.output_field` references from an expression.
///
/// Recursively traverses the expression tree and collects references into
/// a `Vec`.
///
/// # Returns
///
/// A `Vec<(String, String)>`, where each tuple contains:
/// - The name of the referenced component.
/// - The name of the accessed output field.
fn get_component_outputs(expr: &Expr) -> Vec<(String, String)> {
    let mut outputs = Vec::new();
    extract_component_outputs(expr, &mut outputs);
    outputs
}

/// Recursively extracts `component.output_field` references from an expression.
///
/// Uses depth-first traversal to collect references efficiently.
fn extract_component_outputs(expr: &Expr, outputs: &mut Vec<(String, String)>) {
    match expr {
        Expr::Field(ExprField { base, member, .. }) => {
            let field_name = member.to_token_stream().to_string();

            if let Expr::Path(ExprPath { path, .. }) = base.as_ref() {
                if let Some(ident) = path.get_ident() {
                    outputs.push((ident.to_string(), field_name));
                    return;
                }
            }

            if let Some((base, nested)) = extract_single_component_field(base) {
                outputs.push((base, format!("{nested}.{field_name}")));
            }
        }
        Expr::Binary(bin) => {
            extract_component_outputs(&bin.left, outputs);
            extract_component_outputs(&bin.right, outputs);
        }
        Expr::Paren(paren) => extract_component_outputs(&paren.expr, outputs),
        Expr::Call(call) => call
            .args
            .iter()
            .for_each(|arg| extract_component_outputs(arg, outputs)),
        Expr::Tuple(tuple) => tuple
            .elems
            .iter()
            .for_each(|elem| extract_component_outputs(elem, outputs)),
        _ => {}
    }
}

/// Extracts a `(component, field)` pair from a nested field access.
///
/// Converts expressions like `some_component.some_field.sub_value` into
/// `(some_component, "some_field.sub_value")`, preserving full paths.
///
/// # Returns
///
/// An `Option<(String, String)>`:
/// - `Some((component, field))` if a valid field access is found.
/// - `None` otherwise.
fn extract_single_component_field(expr: &Expr) -> Option<(String, String)> {
    if let Expr::Field(ExprField { base, member, .. }) = expr {
        let field_name = member.to_token_stream().to_string();

        if let Expr::Path(ExprPath { path, .. }) = &**base {
            if let Some(ident) = path.get_ident() {
                return Some((ident.to_string(), field_name));
            }
        } else if let Some((base, nested)) = extract_single_component_field(base) {
            return Some((base, format!("{nested}.{field_name}")));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    use itertools::Itertools;
    use petgraph::visit::EdgeRef;
    use quote::quote;
    use syn::{parse2, parse_str};

    #[test]
    fn get_component_outputs_works() {
        let cases = vec![
            (
                quote! { first_house.indoor_temp },
                vec![("first_house", "indoor_temp")],
            ),
            (
                quote! { indoor.occupancy + 10 },
                vec![("indoor", "occupancy")],
            ),
            (
                quote! { building::Thermostat { setpoint: 20.0, auto: false } },
                vec![],
            ),
            (
                quote! { some_component.some_field.sub_value },
                vec![("some_component", "some_field.sub_value")],
            ),
            (
                quote! {
                    some_component.some_field.sub_value
                    + another.output_a
                    + another.output_b
                },
                vec![
                    ("some_component", "some_field.sub_value"),
                    ("another", "output_a"),
                    ("another", "output_b"),
                ],
            ),
        ];

        for (input, expected) in cases {
            let extracted = get_component_outputs(&parse2(input).unwrap());
            let expected: Vec<_> = expected
                .iter()
                .map(|&(comp, field)| (comp.to_string(), field.to_string()))
                .collect();

            assert_eq!(
                extracted, expected,
                "Extracted component outputs did not match expected values."
            );
        }
    }

    #[test]
    fn build_graph_works() -> Result<(), Box<dyn Error>> {
        let graph = build_graph(&[
            ComponentInstance {
                name: parse_str("weather")?,
                component_type: parse_str("hourly_weather")?,
                input_struct: parse_str(
                    r"
                    hourly_weather::Input {
                        time
                    }",
                )?,
            },
            ComponentInstance {
                name: parse_str("first_house")?,
                component_type: parse_str("building")?,
                input_struct: parse_str(
                    r"
                    building::Input { 
                        occupancy: indoor.occupancy,
                        outdoor_temp: weather.temperature,
                        wind_speed: weather.wind_speed,
                        thermostat: building::Thermostat {
                            setpoint: indoor.temp_setpoint,
                            auto: thermostat_control.is_auto,
                        }
                    }",
                )?,
            },
            ComponentInstance {
                name: parse_str("second_house")?,
                component_type: parse_str("building")?,
                input_struct: parse_str(
                    r"
                    building::Input { 
                        occupancy: indoor.occupancy,
                        outdoor_temp: first_house.indoor_temp,
                        wind_speed: weather.wind_speed,
                        thermostat: building::Thermostat {
                            setpoint: indoor.temp_setpoint,
                            auto: thermostat_control.is_auto,
                        }
                    }",
                )?,
            },
            ComponentInstance {
                name: parse_str("another_component")?,
                component_type: parse_str("model")?,
                input_struct: parse_str(
                    r"
                    model::Input { 
                        x: weather.temperature,
                        y: first_house.nested.room_temp,
                        z: second_house.indoor_temp,
                    }",
                )?,
            },
        ]);

        let expected_nodes = vec![0, 1, 2, 3];
        let actual_nodes: Vec<_> = graph.node_indices().map(|idx| graph[idx]).collect();
        assert_eq!(actual_nodes, expected_nodes, "Nodes do not match.");

        let expected_edges = vec![
            (0, 1, "temperature", "outdoor_temp"),
            (0, 1, "wind_speed", "wind_speed"),
            (0, 2, "wind_speed", "wind_speed"),
            (0, 3, "temperature", "x"),
            (1, 2, "indoor_temp", "outdoor_temp"),
            (1, 3, "nested.room_temp", "y"),
            (2, 3, "indoor_temp", "z"),
        ];

        let actual_edges: Vec<_> = graph
            .edge_references()
            .map(|edge| {
                let source = edge.source().index();
                let target = edge.target().index();
                let conn = edge.weight();
                (source, target, conn.source.as_str(), conn.target.as_str())
            })
            .sorted()
            .collect();

        assert_eq!(actual_edges, expected_edges, "Edges do not match.");

        Ok(())
    }
}
