use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Error, Fields, FieldsNamed, Generics, Ident, ItemStruct, Result, Visibility,
    parse::{Parse, ParseStream},
};

use crate::utils::IdentExt;

#[derive(Debug)]
pub(crate) struct Parsed {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    fields: FieldsNamed,
}

impl Parse for Parsed {
    /// Parses a struct definition and validates constraints.
    fn parse(input: ParseStream) -> Result<Self> {
        let ItemStruct {
            vis,
            ident,
            generics,
            fields,
            ..
        } = input.parse()?;

        let Fields::Named(fields) = fields else {
            return Err(Error::new_spanned(
                ident,
                "Unsupported struct type. This macro requires a struct with named fields.",
            ));
        };

        Ok(Parsed {
            vis,
            ident,
            generics,
            fields,
        })
    }
}

impl Parsed {
    /// Generates the full token stream for the macro expansion.
    pub fn expand(self) -> TokenStream {
        let derivatives_struct = self.generate_derivatives_struct();
        let time_integrable_impl = self.generate_time_integrable_impl();

        quote! {
            #derivatives_struct
            #time_integrable_impl
        }
    }

    /// Generates a derivatives struct with `TimeDerivative<T>` for each field,
    /// using the same field names as the original struct.
    fn generate_derivatives_struct(&self) -> TokenStream {
        let vis = &self.vis;
        let deriv_struct_name = self.ident.with_suffix("TimeDerivative");
        let (impl_generics, _ty_generics, where_clause) = self.generics.split_for_impl();

        let derivative_fields: Vec<_> = self
            .fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                quote! {
                    #field_name: twine_core::TimeDerivative<#field_type>
                }
            })
            .collect();

        quote! {
            #[derive(Debug, Clone, PartialEq)]
            #vis struct #deriv_struct_name #impl_generics #where_clause {
                #(#derivative_fields),*
            }
        }
    }

    /// Generates the `TimeIntegrable` implementation.
    fn generate_time_integrable_impl(&self) -> TokenStream {
        let struct_name = &self.ident;
        let deriv_struct_name = self.ident.with_suffix("TimeDerivative");
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let step_assignments: Vec<_> = self
            .fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote! {
                    #field_name: self.#field_name.step(derivative.#field_name, dt)
                }
            })
            .collect();

        quote! {
            impl #impl_generics twine_core::TimeIntegrable for #struct_name #ty_generics #where_clause {
                type Derivative = #deriv_struct_name #ty_generics;

                fn step(self, derivative: Self::Derivative, dt: uom::si::f64::Time) -> Self {
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
            pub struct StateVariables {
                t_first_tank: ThermodynamicTemperature,
                t_second_tank: ThermodynamicTemperature,
            }
        ";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct StateVariablesTimeDerivative {
                t_first_tank: twine_core::TimeDerivative<ThermodynamicTemperature>,
                t_second_tank: twine_core::TimeDerivative<ThermodynamicTemperature>
            }

            impl twine_core::TimeIntegrable for StateVariables {
                type Derivative = StateVariablesTimeDerivative;

                fn step(self, derivative: Self::Derivative, dt: uom::si::f64::Time) -> Self {
                    Self {
                        t_first_tank: self.t_first_tank.step(derivative.t_first_tank, dt),
                        t_second_tank: self.t_second_tank.step(derivative.t_second_tank, dt)
                    }
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn supports_generics() {
        let input = "
            pub(crate) struct State<Fluid: TimeIntegrable> {
                temperature: ThermodynamicTemperature,
                density: MassDensity,
                fluid: Fluid,
            }
        ";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub(crate) struct StateTimeDerivative<Fluid: TimeIntegrable> {
                temperature: twine_core::TimeDerivative<ThermodynamicTemperature>,
                density: twine_core::TimeDerivative<MassDensity>,
                fluid: twine_core::TimeDerivative<Fluid>
            }

            impl<Fluid: TimeIntegrable> twine_core::TimeIntegrable for State<Fluid> {
                type Derivative = StateTimeDerivative<Fluid>;

                fn step(self, derivative: Self::Derivative, dt: uom::si::f64::Time) -> Self {
                    Self {
                        temperature: self.temperature.step(derivative.temperature, dt),
                        density: self.density.step(derivative.density, dt),
                        fluid: self.fluid.step(derivative.fluid, dt)
                    }
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
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
