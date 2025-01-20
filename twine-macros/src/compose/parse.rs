use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::Brace,
    Error, ExprStruct, Ident, PathSegment, Result, Token,
};

use super::{ComponentDefinition, ComponentInstance, InputField, InputSchema};

impl Parse for ComponentDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>().map_err(|_| {
            Error::new(
                input.span(),
                "Expected a valid component name before `{ ... }`.",
            )
        })?;
        input.parse::<Option<Token![,]>>()?;

        let content;
        braced!(content in input);
        let (input_schema, components) = parse_component_body(&content)?;

        Ok(ComponentDefinition {
            name,
            input_schema,
            components,
        })
    }
}

fn parse_component_body(content: ParseStream) -> Result<(InputSchema, Vec<ComponentInstance>)> {
    let mut input_schema = HashMap::new();
    let mut components = Vec::new();
    let mut component_names = HashSet::new();

    while !content.is_empty() {
        let key: Ident = content.parse()?;

        if key == "Input" {
            if !input_schema.is_empty() {
                return Err(Error::new(
                    key.span(),
                    "Multiple `Input {}` blocks found. Only one is allowed.",
                ));
            }
            input_schema = parse_input_fields(content)?;
        } else {
            if !component_names.insert(key.clone()) {
                return Err(Error::new(
                    key.span(),
                    format!("Duplicate component name `{key}` found."),
                ));
            }
            content.parse::<Token![=>]>()?;
            components.push(parse_component_instance(key, content)?);
        }

        content.parse::<Option<Token![,]>>()?;
    }

    Ok((input_schema, components))
}

/// Recursively parses an `InputSchema`, handling nested structures.
fn parse_input_fields(input: ParseStream) -> Result<InputSchema> {
    let content;
    braced!(content in input);

    let mut fields = HashMap::<Ident, InputField>::new();

    while !content.is_empty() {
        let key: Ident = content.parse()?;
        content.parse::<Token![:]>()?;

        if content.peek(Brace) {
            let nested_fields = parse_input_fields(&content)?;
            fields.insert(key, InputField::Struct(nested_fields));
        } else {
            let value = content.parse()?;
            fields.insert(key, InputField::Type(value));
        }

        content.parse::<Option<Token![,]>>()?;
    }

    Ok(fields)
}

/// Parses a single `ComponentInstance`.
fn parse_component_instance(name: Ident, content: ParseStream) -> Result<ComponentInstance> {
    let mut input_struct: ExprStruct = content.parse()?;
    let module = input_struct.path.clone();

    // Append the `::Input` segment to the input struct path.
    input_struct.path.segments.push(PathSegment {
        ident: Ident::new("Input", Span::call_site()),
        arguments: syn::PathArguments::None,
    });

    Ok(ComponentInstance {
        name,
        module,
        input_struct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::ToTokens;
    use syn::parse_str;

    /// Checks if parsing fails with the expected error message.
    fn assert_error_message(input: &str, expected_msg: &str) {
        if let Err(err) = parse_str::<ComponentDefinition>(input) {
            let msg = err.to_string();
            assert!(
                msg.contains(expected_msg),
                "Expected error message containing: {expected_msg}\nGot: {msg}"
            );
        } else {
            panic!("Expected parsing to fail, but it succeeded");
        }
    }

    #[test]
    fn parse_a_component() {
        let input = r"
        example {
            Input {
                time: f64,
                hour: u32,
                indoor: {
                    occupancy: u32,
                }
            }

            house => building {
                occupancy: indoor.occupancy,
                outdoor_temperature: weather.temperature,
            }
        }
        ";

        let component = parse_str::<ComponentDefinition>(input).unwrap();

        assert_eq!(component.name.to_string(), "example");
        assert_eq!(component.input_schema.len(), 3);
        assert!(component
            .input_schema
            .contains_key(&parse_str::<Ident>("time").unwrap()));
        assert!(component
            .input_schema
            .contains_key(&parse_str::<Ident>("hour").unwrap()));
        assert!(component
            .input_schema
            .contains_key(&parse_str::<Ident>("indoor").unwrap()));

        assert_eq!(component.components.len(), 1);
        let house = &component.components[0];
        assert_eq!(house.name.to_string(), "house");
        assert_eq!(house.module.to_token_stream().to_string(), "building");
    }

    #[test]
    fn error_on_missing_component_name() {
        assert_error_message(
            r"
            {
                Input {
                    time: f64
                }
            }
            ",
            "Expected a valid component name before `{ ... }`.",
        );
    }

    #[test]
    fn error_on_multiple_input_blocks() {
        assert_error_message(
            r"
            example {
                Input {
                    time: f64
                }
                Input {
                    hour: u32
                }
            }
            ",
            "Multiple `Input {}` blocks found. Only one is allowed.",
        );
    }

    #[test]
    fn error_on_duplicate_components() {
        assert_error_message(
            r"
            example {
                house => building {
                    temp: 20.0
                }
                house => building {
                    temp: 22.0
                }
            }
            ",
            "Duplicate component name `house` found.",
        );
    }
}
