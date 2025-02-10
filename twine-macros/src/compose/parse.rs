use syn::{
    parse::{Parse, ParseStream},
    Error, Expr, ImplItem, ImplItemFn, ItemImpl, Local, Pat, Path, Result, Stmt, Type,
};

use super::{ComponentDefinition, ComponentInstance};

impl Parse for ComponentDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let item_impl = input.parse()?;

        let component_type = extract_component_type(&item_impl)?;
        let input_type = extract_input_type(&item_impl)?;
        let components = extract_component_instances(&item_impl)?;

        Ok(Self {
            component_type,
            input_type,
            components,
        })
    }
}

/// Extracts the component type from the `impl` block.
fn extract_component_type(item_impl: &ItemImpl) -> Result<Path> {
    match item_impl.self_ty.as_ref() {
        Type::Path(type_path) => Ok(type_path.path.clone()),
        _ => Err(syn::Error::new_spanned(
            item_impl,
            "Expected a valid type path for the component.",
        )),
    }
}

/// Extracts the `Input` type from the `impl` block, ensuring a single occurrence.
fn extract_input_type(item_impl: &ItemImpl) -> Result<Path> {
    let input_types: Vec<Path> = item_impl
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Type(type_item) if type_item.ident == "Input" => match &type_item.ty {
                Type::Path(type_path) => Some(type_path.path.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect();

    match input_types.as_slice() {
        [input_type] => Ok(input_type.clone()),
        [] => Err(Error::new_spanned(
            item_impl,
            "Expected `type Input = SomeType;` inside impl block.",
        )),
        _ => Err(Error::new_spanned(
            item_impl,
            "Multiple `type Input = ...;` definitions found inside impl block.",
        )),
    }
}

/// Extracts component instances from `fn components()`.
fn extract_component_instances(item_impl: &ItemImpl) -> Result<Vec<ComponentInstance>> {
    let fn_statements = find_components_fn(item_impl)?.block.stmts.as_slice();

    fn_statements
        .iter()
        .map(validate_and_extract_instance)
        .collect()
}

/// Finds `fn components()` inside `impl`, ensuring a single occurrence.
fn find_components_fn(item_impl: &ItemImpl) -> Result<&ImplItemFn> {
    let components_fns: Vec<&ImplItemFn> = item_impl
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(method) if method.sig.ident == "components" => Some(method),
            _ => None,
        })
        .collect();

    match components_fns.as_slice() {
        [single] => Ok(single),
        [] => Err(Error::new_spanned(
            item_impl,
            "Expected `fn components()` inside impl block.",
        )),
        _ => Err(Error::new_spanned(
            item_impl,
            "Multiple `fn components()` found inside impl block.",
        )),
    }
}

/// Extracts a component instance from a `let` statement.
fn validate_and_extract_instance(stmt: &Stmt) -> Result<ComponentInstance> {
    if let Stmt::Local(Local {
        pat: Pat::Ident(pat_ident),
        init: Some(init),
        ..
    }) = stmt
    {
        if let Expr::Struct(expr_struct) = init.expr.as_ref() {
            return Ok(ComponentInstance {
                name: pat_ident.ident.clone(),
                component_type: expr_struct.path.clone(),
                input_struct: expr_struct.clone(),
            });
        }
    }

    Err(Error::new_spanned(
        stmt,
        "Only `let name = SomeStruct { ... };` statements are allowed inside `fn components()`.",
    ))
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

    /// Helper to convert any `ToTokens` type to a `String`.
    fn tokens_to_string<T: ToTokens>(item: &T) -> String {
        item.to_token_stream().to_string()
    }

    #[test]
    fn parse_valid_input() {
        let input = "
        impl DemoComponent {
            type Input = DemoInput;

            fn components() {
                let weather = WeatherComponent { time: input.time };
                let house = HouseComponent { temp: weather.temperature };
            }
        }";
        let component = parse_str::<ComponentDefinition>(input).unwrap();

        assert_eq!(tokens_to_string(&component.component_type), "DemoComponent");
        assert_eq!(tokens_to_string(&component.input_type), "DemoInput");

        let [weather, house] = &component.components[..] else {
            panic!("Expected 2 component instances");
        };
        assert_eq!(
            tokens_to_string(&weather.component_type),
            "WeatherComponent"
        );
        assert_eq!(tokens_to_string(&house.component_type), "HouseComponent");
    }

    #[test]
    fn error_on_invalid_component_type() {
        let input = "
        impl (SomeType, OtherType) {
            type Input = DemoInput;

            fn components() {
                let weather = WeatherComponent { time: input.time };
            }
        }";

        assert_error_message(input, "Expected a valid type path for the component.");
    }

    #[test]
    fn error_on_missing_input_type() {
        let input = "
        impl DemoComponent {
            fn components() {
                let first = FirstComponent { foo: input.bar };
            }
        }";

        assert_error_message(
            input,
            "Expected `type Input = SomeType;` inside impl block.",
        );
    }

    #[test]
    fn error_on_multiple_input_type() {
        let input = "
        impl DemoComponent {
            type Input = FirstInput;
            type Input = SecondInput;

            fn components() {
                let first = FirstComponent { foo: input.bar };
            }
        }";

        assert_error_message(
            input,
            "Multiple `type Input = ...;` definitions found inside impl block.",
        );
    }

    #[test]
    fn error_on_missing_components_fn() {
        let input = "
        impl DemoComponent {
            type Input = DemoInput;
        }";

        assert_error_message(input, "Expected `fn components()` inside impl block.");
    }

    #[test]
    fn error_on_multiple_components_fn() {
        let input = "
        impl DemoComponent {
            type Input = DemoInput;

            fn components() {
                let first = FirstComponent { foo: input.bar };
            }

            fn components() {
                let second = SecondComponent { baz: input.qux };
            }
        }";

        assert_error_message(input, "Multiple `fn components()` found inside impl block.");
    }

    #[test]
    fn error_on_invalid_statement_in_components_fn() {
        let input = r#"
        impl DemoComponent {
            type Input = DemoInput;

            fn components() {
                let first = FirstComponent { foo: input.bar };
                println!("This is invalid");
            }
        }"#;

        assert_error_message(input, "Only `let name = SomeStruct { ... };` statements are allowed inside `fn components()`.");
    }
}
