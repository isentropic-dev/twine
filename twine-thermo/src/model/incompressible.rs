use std::convert::Infallible;

use uom::{
    ConstZero,
    si::f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
};

use crate::{
    PropertyError, State,
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy, TemperatureDifference},
};

use super::{StateFrom, ThermodynamicProperties};

/// Trait used to define thermodynamic constants for incompressible fluids.
///
/// This trait provides the fixed properties needed to model a fluid under
/// incompressible assumptions, where density variations are negligible and
/// specific heat is effectively independent of temperature.
///
/// Any type that implements `IncompressibleFluid` can be used with the
/// [`Incompressible`] model to calculate thermodynamic properties like
/// enthalpy, entropy, and specific heats.
///
/// Typically implemented for liquids like [`Water`] or custom incompressible
/// fluids where pressure effects on density can be ignored.
pub trait IncompressibleFluid {
    /// Returns the specific heat capacity.
    fn specific_heat(&self) -> SpecificHeatCapacity;

    /// Returns the reference temperature used in enthalpy and entropy calculations.
    fn reference_temperature(&self) -> ThermodynamicTemperature;

    /// Returns the reference density for this fluid.
    ///
    /// This density typically corresponds to the conditions under which the
    /// constant specific heat was determined.
    /// It serves as the default density when creating states from temperature alone,
    /// though models may override this value in specific contexts.
    fn reference_density(&self) -> MassDensity;

    /// Returns the enthalpy at the reference temperature.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_enthalpy(&self) -> SpecificEnthalpy {
        SpecificEnthalpy::ZERO
    }

    /// Returns the entropy at the reference temperature.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_entropy(&self) -> SpecificEntropy {
        SpecificEntropy::ZERO
    }
}

/// A fluid property model using incompressible liquid assumptions.
///
/// Assumes incompressible behavior and constant specific heat, making it
/// suitable for conditions where pressure and density variations are negligible
/// and specific heat is effectively independent of temperature.
///
/// Provides thermodynamic properties for any fluid that impls `IncompressibleFluid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Incompressible;

impl Incompressible {
    /// Creates a state at the fluid's reference temperature and density.
    #[must_use]
    pub fn reference_state<F: IncompressibleFluid>(fluid: F) -> State<F> {
        let temperature = fluid.reference_temperature();
        let density = fluid.reference_density();

        State {
            temperature,
            density,
            fluid,
        }
    }
}

impl<F: IncompressibleFluid> ThermodynamicProperties<F> for Incompressible {
    /// Pressure is not a thermodynamic property of an incompressible liquid.
    fn pressure(&self, _state: &State<F>) -> Result<Pressure, PropertyError> {
        Err(PropertyError::NotImplemented {
            property: "pressure",
            context: Some(
                "pressure is not a thermodynamic property of an incompressible liquid.".into(),
            ),
        })
    }

    /// Computes internal energy, which is equal to enthalpy for incompressible fluids.
    fn internal_energy(&self, state: &State<F>) -> Result<SpecificInternalEnergy, PropertyError> {
        self.enthalpy(state)
    }

    /// Computes enthalpy using `h = h₀ + c·(T − T₀)`.
    fn enthalpy(&self, state: &State<F>) -> Result<SpecificEnthalpy, PropertyError> {
        let c = state.fluid.specific_heat();
        let t_ref = state.fluid.reference_temperature();
        let h_ref = state.fluid.reference_enthalpy();

        Ok(h_ref + c * state.temperature.minus(t_ref))
    }

    /// Computes entropy with `s = s₀ + c·ln(T/T₀)`.
    fn entropy(&self, state: &State<F>) -> Result<SpecificEntropy, PropertyError> {
        let c = state.fluid.specific_heat();
        let t_ref = state.fluid.reference_temperature();
        let s_ref = state.fluid.reference_entropy();

        Ok(s_ref + c * (state.temperature / t_ref).ln())
    }

    /// Returns the constant specific heat from the fluid.
    fn cp(&self, state: &State<F>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(state.fluid.specific_heat())
    }

    /// Returns the constant specific heat from the fluid.
    fn cv(&self, state: &State<F>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(state.fluid.specific_heat())
    }
}

/// Enables creating incompressible fluid states from temperature alone (uses reference density).
impl<F: IncompressibleFluid + Default> StateFrom<F, ThermodynamicTemperature> for Incompressible {
    type Error = Infallible;

    fn state_from(&self, temperature: ThermodynamicTemperature) -> Result<State<F>, Self::Error> {
        let fluid = F::default();
        let density = fluid.reference_density();

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        specific_heat_capacity::kilojoule_per_kilogram_degree_celsius,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::units::TemperatureDifference;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct MockLiquid;

    impl IncompressibleFluid for MockLiquid {
        fn specific_heat(&self) -> SpecificHeatCapacity {
            SpecificHeatCapacity::new::<kilojoule_per_kilogram_degree_celsius>(10.0)
        }

        fn reference_temperature(&self) -> ThermodynamicTemperature {
            ThermodynamicTemperature::new::<degree_celsius>(25.0)
        }

        fn reference_density(&self) -> MassDensity {
            MassDensity::new::<kilogram_per_cubic_meter>(1.0)
        }
    }

    #[test]
    fn pressure_not_implemented() {
        // State at reference temperature and density.
        let state = Incompressible::reference_state(MockLiquid);

        assert!(Incompressible.pressure(&state).is_err());
    }

    #[test]
    fn internal_energy_equals_enthalpy() -> Result<(), PropertyError> {
        // State at specified temperature and reference density.
        let state: State<MockLiquid> = Incompressible
            .state_from(ThermodynamicTemperature::new::<degree_celsius>(15.0))
            .unwrap();

        let u = Incompressible.internal_energy(&state)?;
        let h = Incompressible.enthalpy(&state)?;
        assert_eq!(u, h);

        Ok(())
    }

    #[test]
    fn increase_temperature() -> Result<(), PropertyError> {
        // State at specified temperature and density.
        let state_a: State<MockLiquid> = Incompressible
            .state_from((
                ThermodynamicTemperature::new::<degree_celsius>(30.0),
                MassDensity::new::<kilogram_per_cubic_meter>(2.0),
            ))
            .unwrap();

        let state_b = state_a
            .clone()
            .with_temperature(ThermodynamicTemperature::new::<degree_celsius>(60.0));

        // Check that enthalpy increases with temperature using `h = h₀ + c·(T - T₀)`.
        let h_a = Incompressible.enthalpy(&state_a)?;
        let h_b = Incompressible.enthalpy(&state_b)?;
        let c = state_a.fluid.specific_heat();
        assert_relative_eq!(
            (h_b - h_a).value,
            (c * state_b.temperature.minus(state_a.temperature)).value,
        );

        // Check that entropy increases with temperature using `s = s₀ + c·ln(T/T₀)`.
        let s_a = Incompressible.entropy(&state_a)?;
        let s_b = Incompressible.entropy(&state_b)?;
        assert_relative_eq!(
            (s_b - s_a).value,
            (c * (state_b.temperature / state_a.temperature).ln()).value,
            epsilon = 1e-10,
        );

        Ok(())
    }
}
