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
/// Each component instance is represented as a node, and edges are created
/// based on field references.
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
/// This function identifies component-to-component dependencies by parsing
/// field expressions and linking a component's input fields to output fields
/// from other components.
///
/// # Arguments
///
/// - `component`: The `ComponentInstance` whose connections are to be extracted.
///
/// # Returns
///
/// A vector of `(component_name, Connection)` pairs, where:
/// - `component_name` is the source component's name.
/// - `Connection` specifies the field mapping from the source output to the
///                current component's input.
fn find_connections(component: &ComponentInstance) -> Vec<(String, Connection)> {
    iter_named_fields(component)
        .flat_map(|(field_name, field_expr)| {
            find_component_outputs(field_expr).into_iter().map(
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
/// Ignores unnamed (tuple) fields, focusing only on explicitly named fields.
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
fn iter_named_fields(component: &ComponentInstance) -> impl Iterator<Item = (String, &Expr)> + '_ {
    component.input_struct.fields.iter().filter_map(|field| {
        if let Member::Named(ident) = &field.member {
            Some((ident.to_string(), &field.expr))
        } else {
            // Ignore tuple fields.
            None
        }
    })
}

/// Extracts all `component.output_field` references from an expression.
///
/// # Arguments
///
/// - `expr`: A reference to an expression (`&Expr`).
///
/// # Returns
///
/// A `Vec<(String, String)>`, where each tuple represents:
/// - The referenced `component` name.
/// - The corresponding `output_field` accessed from that component.
fn find_component_outputs(expr: &Expr) -> Vec<(String, String)> {
    traverse_expression(expr).collect()
}

/// Traverses an expression recursively to collect `(component, field)` references.
fn traverse_expression(expr: &Expr) -> impl Iterator<Item = (String, String)> + '_ {
    let mut results = Vec::new();

    match expr {
        Expr::Field(ExprField { base, member, .. }) => {
            let field_name = member.to_token_stream().to_string();

            if let Expr::Path(ExprPath { path, .. }) = &**base {
                if let Some(ident) = path.get_ident() {
                    results.push((ident.to_string(), field_name));
                }
            } else if let Some((base, nested)) = extract_single_component_field(base) {
                results.push((base, format!("{nested}.{field_name}")));
            }
        }
        Expr::Binary(bin) => {
            results.extend(traverse_expression(&bin.left));
            results.extend(traverse_expression(&bin.right));
        }
        Expr::Paren(paren) => {
            results.extend(traverse_expression(&paren.expr));
        }
        Expr::Call(call) => {
            results.extend(call.args.iter().flat_map(traverse_expression));
        }
        Expr::Tuple(tuple) => {
            results.extend(tuple.elems.iter().flat_map(traverse_expression));
        }
        _ => {}
    }

    results.into_iter()
}

/// Extracts a single `(component, fields)` pair from deeply nested field accesses.
///
/// This function helps break down nested field references like
/// `some_component.some_field.sub_value` into a structured `(component,
/// "some_field.sub_value")` format.
///
/// # Arguments
///
/// - `expr`: A reference to an expression (`&Expr`).
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

    fn expected_map(entries: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        entries
            .iter()
            .map(|&(key, values)| (key.into(), values.iter().map(|&v| v.into()).collect()))
            .collect()
    }

    #[test]
    fn find_component_outputs_works() {
        let cases = vec![
            (
                quote! { indoor.occupancy },
                vec![("indoor".to_string(), "occupancy".to_string())],
            ),
            (
                quote! { first_house.indoor_temp },
                vec![("first_house".to_string(), "indoor_temp".to_string())],
            ),
            (
                quote! { building::Thermostat { setpoint: 20.0, auto: false } },
                vec![],
            ),
            (
                quote! { some_component.some_field.sub_value },
                vec![(
                    "some_component".to_string(),
                    "some_field.sub_value".to_string(),
                )],
            ),
            (
                quote! {
                    some_component.some_field.sub_value
                    + another.output_a
                    + another.output_b
                },
                vec![
                    (
                        "some_component".to_string(),
                        "some_field.sub_value".to_string(),
                    ),
                    ("another".to_string(), "output_a".to_string()),
                    ("another".to_string(), "output_b".to_string()),
                ],
            ),
        ];

        for (input_expr, expected) in cases {
            let extracted = find_component_outputs(&parse2::<Expr>(input_expr).unwrap());

            assert_eq!(
                extracted, expected,
                "Extracted input references did not match expected values."
            );
        }
    }

    #[test]
    fn build_a_graph() -> Result<(), Box<dyn Error>> {
        let graph = build_graph(&[
            ComponentInstance {
                name: parse_str("weather")?,
                module: parse_str("hourly_weather")?,
                input_struct: parse_str(
                    "hourly_weather::Input {
                                 time
                             }",
                )?,
            },
            ComponentInstance {
                name: parse_str("first_house")?,
                module: parse_str("building")?,
                input_struct: parse_str(
                    "building::Input { 
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
                    "building::Input { 
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
                    "model::Input { 
                    x: weather.temperature,
                    y: first_house.nested.room_temp,
                    z: second_house.indoor_temp,
                         }",
                )?,
            },
        ]);

        let expected_nodes = vec![0, 1, 2, 3];
        let actual_nodes: Vec<_> = graph.node_indices().map(|idx| graph[idx]).collect();
        assert_eq!(
            actual_nodes, expected_nodes,
            "Graph nodes do not match expected indices."
        );

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

        assert_eq!(
            actual_edges, expected_edges,
            "Graph edges do not match expected dependencies"
        );

        Ok(())
    }
}
