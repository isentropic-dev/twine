#![allow(dead_code)] // Just for now...

use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};
use quote::ToTokens;
use syn::{Expr, ExprField, ExprPath, Member};

use super::ComponentInstance;

/// Represents a directed connection between components in the graph.
///
/// - `from_output`: The name of the output field of the source component.
/// - `to_input`: The name of the input field of the destination component.
#[derive(Debug, PartialEq)]
pub(crate) struct Connection {
    from_output: String,
    to_input: String,
}

/// Builds a directed dependency graph from a list of component instances.
///
/// Each component instance is represented as a node in the graph, indexed by
/// its position in the input list (`components`). Directed edges represent
/// dependencies between components based on their input structures.
///
/// How It Works:
///
/// - Each component has an input struct that defines its field assignments.
/// - If a field references another component's output (e.g., if `first_house`
///   has an input field using `weather.temperature`), a directed edge is
///   created from `weather` to `first_house`.
/// - Nested field accesses (e.g., `component_name.with.nested.output`) are
///   stored as `"with.nested.output"` in the graph, preserving the full path
///   to the output.
/// - Some input fields resemble component names but are actually inputs to a
///   larger composed system (defined by the macro). These are not treated as
///   dependencies and are ignored when constructing the graph.
///
/// Returns:
///
/// A `DiGraph<usize, Connection>` where:
/// - Nodes are component indices.
/// - Edges represent dependencies between components, labeled with `Connection`.
pub(crate) fn build_graph(components: &[ComponentInstance]) -> DiGraph<usize, Connection> {
    let mut graph = DiGraph::new();

    let node_map: HashMap<String, NodeIndex> = components
        .iter()
        .enumerate()
        .map(|(index, component)| (component.name.to_string(), graph.add_node(index)))
        .collect();

    for (to_index, component) in components.iter().enumerate() {
        for (from_component, connection) in find_connections(component) {
            if let Some(&from_index) = node_map.get(&from_component) {
                graph.add_edge(from_index, NodeIndex::new(to_index), connection);
            }
        }
    }

    graph
}

/// Extracts connections for a component based on its input field expressions.
///
/// This function determines dependencies between components by parsing input
/// field expressions and linking each input to an output field from another
/// component.
///
/// # Arguments
///
/// - `component`: The `ComponentInstance` whose connections are being extracted.
///
/// # Returns
///
/// A vector of `(component_name, Connection)` pairs, where:
/// - `component_name` is the source component's name.
/// - `Connection` maps the source component's output to this component's input.
fn find_connections(component: &ComponentInstance) -> Vec<(String, Connection)> {
    iter_named_fields(component)
        .flat_map(|(field_name, field_expr)| {
            get_component_outputs(field_expr).into_iter().map(
                move |(component_name, output_name)| {
                    (
                        component_name,
                        Connection {
                            from_output: output_name,
                            to_input: field_name.clone(),
                        },
                    )
                },
            )
        })
        .collect()
}

/// Iterates over the named fields of a component.
///
/// Only explicitly named fields are returned, while unnamed (tuple) fields are ignored.
///
/// # Arguments
///
/// - `component`: A reference to a `ComponentInstance`.
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

/// Gets all `component.output_field` references from an expression.
///
/// Initializes an empty `Vec` and recursively traverses the expression
/// tree using `extract_component_outputs`.
///
/// # Arguments
///
/// - `expr`: The expression to analyze.
///
/// # Returns
///
/// A `Vec<(String, String)>`, where each tuple contains:
/// - The referenced component's name.
/// - The accessed output field.
fn get_component_outputs(expr: &Expr) -> Vec<(String, String)> {
    let mut outputs = Vec::new();
    extract_component_outputs(expr, &mut outputs);
    outputs
}

/// Recursively extracts `component.output_field` references into `outputs`.
///
/// Performs a depth-first traversal of the expression tree, collecting
/// component output references into a single `Vec` to minimize allocations.
///
/// # Arguments
///
/// - `expr`: The expression to analyze.
/// - `outputs`: A mutable `Vec` that stores the extracted output references.
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

/// Extracts a single `(component, field)` pair from a deeply nested field access.
///
/// Converts a reference like `some_component.some_field.sub_value` into a
/// structured `(component, "some_field.sub_value")` format.
///
/// # Arguments
///
/// - `expr`: The expression (`&Expr`) to analyze.
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
                module: parse_str("hourly_weather")?,
                input_struct: parse_str(
                    r"
                    hourly_weather::Input {
                        time
                    }",
                )?,
            },
            ComponentInstance {
                name: parse_str("first_house")?,
                module: parse_str("building")?,
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
                module: parse_str("building")?,
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
                module: parse_str("model")?,
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
        assert_eq!(actual_nodes, expected_nodes);

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
                let from = graph[edge.source()];
                let to = graph[edge.target()];
                let conn = edge.weight();
                (from, to, conn.from_output.as_str(), conn.to_input.as_str())
            })
            .sorted()
            .collect();

        assert_eq!(actual_edges, expected_edges);

        Ok(())
    }
}
