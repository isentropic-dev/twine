use twine_core::TimeIntegrable;
use uom::si::f64::{Pressure, SpecificHeatCapacity};

use crate::{
    IncompressibleProperties, PropertyError, State, ThermodynamicProperties,
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy, TemperatureDifference},
};

/// A fluid property model using incompressible liquid assumptions.
///
/// Assumes incompressible behavior and constant specific heat, making it
/// suitable for conditions where pressure and density variations are negligible
/// and specific heat is effectively independent of temperature.
#[derive(Debug, Clone, PartialEq)]
pub struct Incompressible;

impl<F> ThermodynamicProperties<F> for Incompressible
where
    F: TimeIntegrable + IncompressibleProperties,
{
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
