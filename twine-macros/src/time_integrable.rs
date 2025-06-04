use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Error, Fields, FieldsNamed, Ident, ItemStruct, Result,
};

use crate::utils::IdentExt;

#[derive(Debug)]
pub(crate) struct Parsed {
    ident: Ident,
    fields: FieldsNamed,
}

impl Parse for Parsed {
    /// Parses a struct definition and validates constraints.
    fn parse(input: ParseStream) -> Result<Self> {
        let ItemStruct {
            ident,
            generics,
            fields,
            ..
        } = input.parse()?;

        if !generics.params.is_empty() {
            return Err(Error::new_spanned(
                generics,
                "Generic parameters are not allowed. Remove them to use this macro.",
            ));
        }

        let Fields::Named(fields) = fields else {
            return Err(Error::new_spanned(
                ident,
                "Unsupported struct type. This macro requires a struct with named fields.",
            ));
        };

        Ok(Parsed { ident, fields })
    }
}

impl Parsed {
    /// Generates the full token stream for the macro expansion.
    pub fn expand(self) -> TokenStream {
        let derivatives_struct = self.generate_derivatives_struct();
        let div_impl = self.generate_div_impl();
        let time_integrable_impl = self.generate_time_integrable_impl();

        quote! {
            #derivatives_struct
            #div_impl
            #time_integrable_impl
        }
    }

    /// Generates a derivatives struct with `TimeDerivativeOf<T>` for each field.
    fn generate_derivatives_struct(&self) -> TokenStream {
        let deriv_struct_name = self.ident.with_suffix("Dt");

        let derivative_fields: Vec<_> = self
            .fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap().with_suffix("_dt");
                let field_type = &field.ty;
                quote! {
                    #field_name: twine_core::TimeDerivativeOf<#field_type>
                }
            })
            .collect();

        quote! {
            struct #deriv_struct_name {
                #(#derivative_fields),*
            }
        }
    }

    /// Generates the `Div<Time>` implementation for the original struct.
    fn generate_div_impl(&self) -> TokenStream {
        let struct_name = &self.ident;
        let deriv_struct_name = self.ident.with_suffix("Dt");

        let derivative_assignments: Vec<_> = self
            .fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let derivative_name = field_name.with_suffix("_dt");
                quote! {
                    #derivative_name: self.#field_name / rhs
                }
            })
            .collect();

        quote! {
            impl std::ops::Div<uom::si::f64::Time> for #struct_name {
                type Output = #deriv_struct_name;

                fn div(self, rhs: uom::si::f64::Time) -> Self::Output {
                    Self::Output {
                        #(#derivative_assignments),*
                    }
                }
            }
        }
    }

    /// Generates the `TimeIntegrable` implementation for the original struct.
    fn generate_time_integrable_impl(&self) -> TokenStream {
        let struct_name = &self.ident;
        let deriv_struct_name = self.ident.with_suffix("Dt");

        let step_assignments: Vec<_> = self
            .fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let derivative_name = field_name.with_suffix("_dt");
                quote! {
                    #field_name: self.#field_name + derivative.#derivative_name * dt
                }
            })
            .collect();

        quote! {
            impl twine_core::TimeIntegrable for #struct_name {
                fn step_by_time(self, derivative: #deriv_struct_name, dt: uom::si::f64::Time) -> Self {
                    Self {
                        #(#step_assignments),*
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn generates_correct_code() {
        let input = "
            struct StateVariables {
                t_first_tank: ThermodynamicTemperature,
                t_second_tank: ThermodynamicTemperature,
            }
        ";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            struct StateVariablesDt {
                t_first_tank_dt: twine_core::TimeDerivativeOf<ThermodynamicTemperature>,
                t_second_tank_dt: twine_core::TimeDerivativeOf<ThermodynamicTemperature>
            }

            impl std::ops::Div<uom::si::f64::Time> for StateVariables {
                type Output = StateVariablesDt;

                fn div(self, rhs: uom::si::f64::Time) -> Self::Output {
                    Self::Output {
                        t_first_tank_dt: self.t_first_tank / rhs,
                        t_second_tank_dt: self.t_second_tank / rhs
                    }
                }
            }

            impl twine_core::TimeIntegrable for StateVariables {
                fn step_by_time(self, derivative: StateVariablesDt, dt: uom::si::f64::Time) -> Self {
                    Self {
                        t_first_tank: self.t_first_tank + derivative.t_first_tank_dt * dt,
                        t_second_tank: self.t_second_tank + derivative.t_second_tank_dt * dt
                    }
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn error_if_generics_are_present() {
        let error_message = parse_str::<Parsed>(
            "struct StateWithGenerics<T> {
                value: T,
            }",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error_message,
            "Generic parameters are not allowed. Remove them to use this macro."
        );
    }

    #[test]
    fn error_if_tuple_struct() {
        let error_message = parse_str::<Parsed>("struct TupleState(f64, f64);")
            .unwrap_err()
            .to_string();

        assert_eq!(
            error_message,
            "Unsupported struct type. This macro requires a struct with named fields."
        );
    }
}
