use proc_macro2::Span;
use syn::{
    Attribute, Error, Ident, ItemFn, Result, ReturnType, Stmt, Visibility,
    parse::{Parse, ParseStream},
};

/// Represents the fully parsed macro input.
#[derive(Debug)]
pub(crate) struct Parsed {
    pub(crate) name: Ident,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) vis: Visibility,
    pub(crate) stmts: Vec<Stmt>,
}

impl Parsed {
    pub(crate) fn new(parsed_attr: ParsedAttr, parsed_item: ParsedItem) -> Self {
        let ParsedAttr { name } = parsed_attr;
        let ParsedItem { attrs, vis, stmts } = parsed_item;
        Self {
            name,
            attrs,
            vis,
            stmts,
        }
    }
}

/// Represents the parsed `#[compose(MyComponent)]` attribute.
#[derive(Debug)]
pub(crate) struct ParsedAttr {
    name: Ident,
}

impl Parse for ParsedAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "Expected a name for the composed component (e.g., `#[compose(MyComponent)]`).",
            ));
        }

        Ok(Self {
            name: input.parse()?,
        })
    }
}

/// Represents the parsed `fn compose()` function.
#[derive(Debug)]
pub(crate) struct ParsedItem {
    attrs: Vec<Attribute>,
    vis: Visibility,
    stmts: Vec<Stmt>,
}

impl Parse for ParsedItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let ItemFn {
            attrs,
            vis,
            sig,
            block,
        } = input.parse()?;

        if sig.ident != "compose" {
            return Err(Error::new_spanned(
                sig.ident,
                "Expected function name to be `compose`.",
            ));
        }

        if !sig.generics.params.is_empty() {
            return Err(Error::new_spanned(
                sig.generics,
                "Unexpected generic parameters.",
            ));
        }

        if !sig.inputs.is_empty() {
            return Err(Error::new_spanned(
                sig.inputs,
                "Unexpected function parameters.",
            ));
        }

        if sig.output != ReturnType::Default {
            return Err(Error::new_spanned(sig.output, "Unexpected return type."));
        }

        Ok(Self {
            attrs,
            vis,
            stmts: block.stmts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn parses_valid_attr() {
        let parsed = parse_str::<ParsedAttr>("MyComponent").expect("Parsing should succeed");
        assert_eq!(parsed.name, "MyComponent");
    }

    #[test]
    fn error_if_attr_is_empty() {
        let error_message = parse_str::<ParsedAttr>("").unwrap_err().to_string();

        assert_eq!(
            error_message,
            "Expected a name for the composed component (e.g., `#[compose(MyComponent)]`)."
        );
    }

    #[test]
    fn parses_a_valid_compose_function() {
        let parsed = parse_str::<ParsedItem>(
            "fn compose() {
                type Input = Input;
                type Components = Components;

                Connections {
                    first: input.x,
                    second: output.first,
                }
            }",
        )
        .expect("Parsing should succeed");

        assert_eq!(
            parsed.stmts.len(),
            3,
            "Expected three statements in the function body."
        );
    }

    #[test]
    fn error_if_function_name_is_wrong() {
        let error_message = parse_str::<ParsedItem>(
            "fn not_compose() {
                let x = 42;
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error_message, "Expected function name to be `compose`.");
    }

    #[test]
    fn error_if_generics_are_present() {
        let error_message = parse_str::<ParsedItem>(
            "fn compose<T>() {
                let x = 42;
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error_message, "Unexpected generic parameters.");
    }

    #[test]
    fn error_if_parameters_are_present() {
        let error_message = parse_str::<ParsedItem>(
            "fn compose(x: i32) {
                let y = x + 1;
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error_message, "Unexpected function parameters.");
    }

    #[test]
    fn error_if_return_type_is_present() {
        let error_message = parse_str::<ParsedItem>(
            "fn compose() -> i32 {
                42
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error_message, "Unexpected return type.");
    }
}
