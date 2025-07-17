use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Generics, Ident, Item, Result, Visibility,
    parse::{Parse, ParseStream},
};

use crate::utils::IdentExt;

#[derive(Debug)]
pub(crate) struct Parsed {
    ident: Ident,
    generics: Generics,
    vis: Visibility,
}

impl Parse for Parsed {
    /// Parses a struct or enum definition.
    fn parse(input: ParseStream) -> Result<Self> {
        let item: Item = input.parse()?;
        
        let (ident, generics, vis) = match item {
            Item::Struct(item_struct) => (item_struct.ident, item_struct.generics, item_struct.vis),
            Item::Enum(item_enum) => (item_enum.ident, item_enum.generics, item_enum.vis),
            _ => return Err(syn::Error::new_spanned(
                item,
                "TimeIndependent can only be derived for structs and enums"
            )),
        };

        Ok(Parsed { ident, generics, vis })
    }
}

impl Parsed {
    /// Generates the full token stream for the macro expansion.
    pub fn expand(self) -> TokenStream {
        let derivative_struct = self.generate_derivative_struct();
        let delta_struct = self.generate_delta_struct();
        let div_impl = self.generate_div_impl();
        let mul_impl = self.generate_mul_impl();
        let add_impl = self.generate_add_impl();

        quote! {
            #derivative_struct
            #delta_struct
            #div_impl
            #mul_impl
            #add_impl
        }
    }

    /// Generates a zero-sized derivative struct.
    fn generate_derivative_struct(&self) -> TokenStream {
        let derivative_struct_name = self.ident.with_suffix("TimeDerivative");
        let generics = &self.generics;
        let vis = &self.vis;

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            #vis struct #derivative_struct_name #generics;
        }
    }

    /// Generates a zero-sized delta struct.
    fn generate_delta_struct(&self) -> TokenStream {
        let delta_struct_name = self.ident.with_suffix("TimeDelta");
        let generics = &self.generics;
        let vis = &self.vis;

        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            #vis struct #delta_struct_name #generics;
        }
    }

    /// Generates the `Div<Time>` implementation for the original struct.
    fn generate_div_impl(&self) -> TokenStream {
        let struct_name = &self.ident;
        let derivative_struct_name = self.ident.with_suffix("TimeDerivative");
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_generics std::ops::Div<uom::si::f64::Time> for #struct_name #ty_generics #where_clause {
                type Output = #derivative_struct_name #ty_generics;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    #derivative_struct_name
                }
            }
        }
    }

    /// Generates the `Mul<Time>` implementation for the derivative struct.
    fn generate_mul_impl(&self) -> TokenStream {
        let derivative_struct_name = self.ident.with_suffix("TimeDerivative");
        let delta_struct_name = self.ident.with_suffix("TimeDelta");
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_generics std::ops::Mul<uom::si::f64::Time> for #derivative_struct_name #ty_generics #where_clause {
                type Output = #delta_struct_name #ty_generics;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    #delta_struct_name
                }
            }
        }
    }

    /// Generates the `Add<Delta>` implementation for the original struct.
    fn generate_add_impl(&self) -> TokenStream {
        let struct_name = &self.ident;
        let delta_struct_name = self.ident.with_suffix("TimeDelta");
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_generics std::ops::Add<#delta_struct_name #ty_generics> for #struct_name #ty_generics #where_clause {
                type Output = Self;

                fn add(self, _rhs: #delta_struct_name #ty_generics) -> Self::Output {
                    self
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
    fn generates_correct_code_for_named_struct() {
        let input = "
            struct MyState {
                temperature: f64,
                pressure: f64,
            }
        ";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct MyStateTimeDerivative;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct MyStateTimeDelta;

            impl std::ops::Div<uom::si::f64::Time> for MyState {
                type Output = MyStateTimeDerivative;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    MyStateTimeDerivative
                }
            }

            impl std::ops::Mul<uom::si::f64::Time> for MyStateTimeDerivative {
                type Output = MyStateTimeDelta;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    MyStateTimeDelta
                }
            }

            impl std::ops::Add<MyStateTimeDelta> for MyState {
                type Output = Self;

                fn add(self, _rhs: MyStateTimeDelta) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generates_correct_code_for_tuple_struct() {
        let input = "struct TupleState(f64, f64);";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct TupleStateTimeDerivative;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct TupleStateTimeDelta;

            impl std::ops::Div<uom::si::f64::Time> for TupleState {
                type Output = TupleStateTimeDerivative;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    TupleStateTimeDerivative
                }
            }

            impl std::ops::Mul<uom::si::f64::Time> for TupleStateTimeDerivative {
                type Output = TupleStateTimeDelta;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    TupleStateTimeDelta
                }
            }

            impl std::ops::Add<TupleStateTimeDelta> for TupleState {
                type Output = Self;

                fn add(self, _rhs: TupleStateTimeDelta) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generates_correct_code_with_generics() {
        let input = "struct GenericState<T> { value: T }";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct GenericStateTimeDerivative<T>;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct GenericStateTimeDelta<T>;

            impl<T> std::ops::Div<uom::si::f64::Time> for GenericState<T> {
                type Output = GenericStateTimeDerivative<T>;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    GenericStateTimeDerivative
                }
            }

            impl<T> std::ops::Mul<uom::si::f64::Time> for GenericStateTimeDerivative<T> {
                type Output = GenericStateTimeDelta<T>;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    GenericStateTimeDelta
                }
            }

            impl<T> std::ops::Add<GenericStateTimeDelta<T> > for GenericState<T> {
                type Output = Self;

                fn add(self, _rhs: GenericStateTimeDelta<T>) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generates_correct_code_for_unit_struct() {
        let input = "struct UnitState;";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct UnitStateTimeDerivative;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct UnitStateTimeDelta;

            impl std::ops::Div<uom::si::f64::Time> for UnitState {
                type Output = UnitStateTimeDerivative;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    UnitStateTimeDerivative
                }
            }

            impl std::ops::Mul<uom::si::f64::Time> for UnitStateTimeDerivative {
                type Output = UnitStateTimeDelta;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    UnitStateTimeDelta
                }
            }

            impl std::ops::Add<UnitStateTimeDelta> for UnitState {
                type Output = Self;

                fn add(self, _rhs: UnitStateTimeDelta) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generates_correct_code_for_enum() {
        let input = "enum MyEnum { A, B(i32), C { x: f64 } }";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct MyEnumTimeDerivative;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct MyEnumTimeDelta;

            impl std::ops::Div<uom::si::f64::Time> for MyEnum {
                type Output = MyEnumTimeDerivative;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    MyEnumTimeDerivative
                }
            }

            impl std::ops::Mul<uom::si::f64::Time> for MyEnumTimeDerivative {
                type Output = MyEnumTimeDelta;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    MyEnumTimeDelta
                }
            }

            impl std::ops::Add<MyEnumTimeDelta> for MyEnum {
                type Output = Self;

                fn add(self, _rhs: MyEnumTimeDelta) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generates_correct_code_for_generic_enum() {
        let input = "enum Result<T, E> { Ok(T), Err(E) }";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct ResultTimeDerivative<T, E>;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            struct ResultTimeDelta<T, E>;

            impl<T, E> std::ops::Div<uom::si::f64::Time> for Result<T, E> {
                type Output = ResultTimeDerivative<T, E>;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    ResultTimeDerivative
                }
            }

            impl<T, E> std::ops::Mul<uom::si::f64::Time> for ResultTimeDerivative<T, E> {
                type Output = ResultTimeDelta<T, E>;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    ResultTimeDelta
                }
            }

            impl<T, E> std::ops::Add<ResultTimeDelta<T, E> > for Result<T, E> {
                type Output = Self;

                fn add(self, _rhs: ResultTimeDelta<T, E>) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn generates_correct_code_for_public_struct() {
        let input = "pub struct PublicState { value: i32 }";

        let parsed = parse_str::<Parsed>(input).expect("Parsing should succeed");
        let generated_code = parsed.expand();

        let expected_code = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            pub struct PublicStateTimeDerivative;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
            pub struct PublicStateTimeDelta;

            impl std::ops::Div<uom::si::f64::Time> for PublicState {
                type Output = PublicStateTimeDerivative;

                fn div(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    PublicStateTimeDerivative
                }
            }

            impl std::ops::Mul<uom::si::f64::Time> for PublicStateTimeDerivative {
                type Output = PublicStateTimeDelta;

                fn mul(self, _rhs: uom::si::f64::Time) -> Self::Output {
                    PublicStateTimeDelta
                }
            }

            impl std::ops::Add<PublicStateTimeDelta> for PublicState {
                type Output = Self;

                fn add(self, _rhs: PublicStateTimeDelta) -> Self::Output {
                    self
                }
            }
        };

        assert_eq!(generated_code.to_string(), expected_code.to_string());
    }

    #[test]
    fn error_for_unsupported_item() {
        let input = "fn my_function() {}";

        let error_message = parse_str::<Parsed>(input)
            .unwrap_err()
            .to_string();

        assert!(error_message.contains("TimeIndependent can only be derived for structs and enums"));
    }
}