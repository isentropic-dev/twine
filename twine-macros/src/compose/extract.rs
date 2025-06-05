use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Expr, ExprStruct, Item, ItemType, Stmt, Type, TypePath, parse_quote, spanned::Spanned};

use crate::utils::IdentExt;

use super::generate_connections_type;

/// Represents data extracted from the parsed macro input.
#[derive(Debug, Clone)]
pub(crate) struct Extracted {
    pub(crate) input: TypePath,
    pub(crate) components: Components,
    pub(crate) connections: ExprStruct,
}

/// Represents the `[composable]` component types used by the macro.
#[derive(Debug, Clone)]
pub(crate) struct Components {
    pub(crate) template: TypePath,
    pub(crate) types_trait: TypePath,
    pub(crate) concrete: TypePath,
    pub(crate) inputs: TypePath,
    pub(crate) outputs: TypePath,
}

impl Components {
    /// Retrieves the name of the components template.
    pub(crate) fn template_name(&self) -> String {
        self.template
            .path
            .segments
            .last()
            .expect("A `TypePath` always has at least one segment.")
            .ident
            .to_string()
    }
}

impl From<TypePath> for Components {
    fn from(template: TypePath) -> Self {
        let types_trait = append_path_suffix(&template, "Types");

        let concrete = parse_quote! {
            <() as #types_trait>::__Concrete
        };

        let inputs = parse_quote! {
            <() as #types_trait>::__Inputs
        };

        let outputs = parse_quote! {
            <() as #types_trait>::__Outputs
        };

        Self {
            template,
            types_trait,
            concrete,
            inputs,
            outputs,
        }
    }
}

impl TryFrom<&[Stmt]> for Extracted {
    type Error = TokenStream;

    fn try_from(stmts: &[Stmt]) -> Result<Self, Self::Error> {
        let (input, components) = extract_types(stmts)?;
        let connections = extract_connections(stmts, &components)?;

        Ok(Self {
            input,
            components,
            connections,
        })
    }
}

/// Extracts the required type aliases (`Input` and `Components`).
fn extract_types(stmts: &[Stmt]) -> Result<(TypePath, Components), TokenStream> {
    let input = extract_type_path(stmts.first(), "Input");
    let components = extract_type_path(stmts.get(1), "Components");

    match (input, components) {
        // Successfully extracted both type paths.
        (Ok(input), Ok(components)) => Ok((input, components.into())),

        // Failed to extract both type paths.
        (Err(err1), Err(err2)) => Err(quote! { #err1 #err2 }),

        // Failed to extract one of the type paths.
        (Err(err), _) | (_, Err(err)) => Err(err),
    }
}

/// Extracts the `Connections` struct expression that defines component wiring.
fn extract_connections(stmts: &[Stmt], components: &Components) -> Result<ExprStruct, TokenStream> {
    let err_message = "Expected a `Connections { ... }` struct expression.";

    let Some(stmt) = stmts.get(2) else {
        return Err(quote! {
            compile_error!(#err_message);
        });
    };

    let Stmt::Expr(Expr::Struct(expr_struct), _) = stmt.clone() else {
        let connections_type = generate_connections_type(components);

        return Err(quote_spanned!(stmt.span() =>
            fn __lsp_support() {
                #connections_type
                let _: Connections = #stmt;
                compile_error!(#err_message);
            }
        ));
    };

    Ok(expr_struct)
}

fn extract_type_path(stmt: Option<&Stmt>, type_name: &str) -> Result<TypePath, TokenStream> {
    let err_message = format!("Expected `type {type_name} = My{type_name};`.");

    let Some(stmt) = stmt else {
        return Err(quote! {
            compile_error!(#err_message);
        });
    };

    let Stmt::Item(Item::Type(ItemType { ident, ty, .. })) = stmt else {
        return Err(quote_spanned!(stmt.span() =>
            compile_error!(#err_message);
        ));
    };

    if ident != type_name {
        return Err(quote_spanned!(ident.span() =>
            compile_error!(#err_message);
        ));
    }

    let Type::Path(type_path) = ty.as_ref() else {
        let err = format!("The `{type_name}` type must be a valid path or identifier.");
        return Err(quote_spanned!(ty.span() =>
            compile_error!(#err);
        ));
    };

    Ok(type_path.clone())
}

/// Appends a suffix to the final segment of a `TypePath`.
fn append_path_suffix(type_path: &TypePath, suffix: &str) -> TypePath {
    let mut type_path = type_path.clone();

    let last = type_path
        .path
        .segments
        .last_mut()
        .expect("A `TypePath` always has at least one segment.");

    last.ident = last.ident.with_suffix(suffix);

    type_path
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::quote;

    #[test]
    fn extract_types_and_connetions() {
        let stmts: Vec<Stmt> = parse_quote! {
            type Input = MyInput;
            type Components = MyComponents;

            Connections {
                first: input.value,
                second: output.first,
            }
        };

        let (input, components) = extract_types(&stmts).unwrap();

        assert_eq!(input, parse_quote!(MyInput));
        assert_eq!(components.template, parse_quote!(MyComponents));

        let connections = extract_connections(&stmts, &components).unwrap();

        assert_eq!(
            connections,
            parse_quote!(Connections {
                first: input.value,
                second: output.first,
            }),
        );
    }

    #[test]
    fn error_if_missing_both_types() {
        let stmts: Vec<Stmt> = vec![parse_quote!(
            let x = 1;
        )];

        assert_eq!(
            extract_types(&stmts).unwrap_err().to_string(),
            quote! {
                compile_error!("Expected `type Input = MyInput;`.");
                compile_error!("Expected `type Components = MyComponents;`.");
            }
            .to_string()
        );
    }

    #[test]
    fn error_if_missing_components_type() {
        let stmts: Vec<Stmt> = vec![parse_quote!(
            type Input = MyInput;
        )];

        assert_eq!(
            extract_types(&stmts).unwrap_err().to_string(),
            quote! {
                compile_error!("Expected `type Components = MyComponents;`.");
            }
            .to_string()
        );
    }

    #[test]
    fn error_if_missing_connections_struct() {
        let stmts: Vec<Stmt> = vec![
            parse_quote!(
                type Input = MyInput;
            ),
            parse_quote!(
                type Components = MyComponents;
            ),
        ];

        let (_input, components) = extract_types(&stmts).unwrap();

        assert_eq!(
            extract_connections(&stmts, &components)
                .unwrap_err()
                .to_string(),
            quote! {
                compile_error!("Expected a `Connections { ... }` struct expression.");
            }
            .to_string()
        );
    }
}
