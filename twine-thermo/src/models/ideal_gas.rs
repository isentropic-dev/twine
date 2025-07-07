use twine_core::TimeIntegrable;
use uom::si::f64::{Pressure, SpecificHeatCapacity};

use crate::{
    IdealGasProperties, PropertyError, State, ThermodynamicProperties,
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy, TemperatureDifference},
};

/// A fluid property model using ideal gas assumptions.
///
/// Assumes ideal gas behavior and constant specific heat, making it
/// suitable for conditions where real gas effects are negligible.
#[derive(Debug, Clone, PartialEq)]
pub struct IdealGas;

impl<F> ThermodynamicProperties<F> for IdealGas
where
    F: TimeIntegrable + IdealGasProperties,
{
    /// Computes pressure with `P = ρ·R·T`.
    fn pressure(&self, state: &State<F>) -> Result<Pressure, PropertyError> {
        Ok(state.density * state.fluid.gas_constant() * state.temperature)
    }

    /// Computes internal energy with `u = h − R·T`.
    fn internal_energy(&self, state: &State<F>) -> Result<SpecificInternalEnergy, PropertyError> {
        Ok(self.enthalpy(state)? - state.fluid.gas_constant() * state.temperature)
    }

    /// Computes enthalpy with `h = h₀ + cp·(T − T₀)`.
    fn enthalpy(&self, state: &State<F>) -> Result<SpecificEnthalpy, PropertyError> {
        let cp = state.fluid.cp();
        let t_ref = state.fluid.reference_temperature();
        let h_ref = state.fluid.reference_enthalpy();

        Ok(h_ref + cp * state.temperature.minus(t_ref))
    }

    /// Computes entropy with `s = s₀ + cp·ln(T⁄T₀) − R·ln(p⁄p₀)`.
    fn entropy(&self, state: &State<F>) -> Result<SpecificEntropy, PropertyError> {
        let cp = state.fluid.cp();
        let r = state.fluid.gas_constant();
        let t_ref = state.fluid.reference_temperature();
        let p_ref = state.fluid.reference_pressure();
        let s_ref = state.fluid.reference_entropy();

        let p = self.pressure(state)?;

        Ok(s_ref + cp * (state.temperature / t_ref).ln() - r * (p / p_ref).ln())
    }

    /// Returns the constant `cp` from the fluid.
    fn cp(&self, state: &State<F>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(state.fluid.cp())
    }

    /// Computes the constant `cv = cp − R`.
    fn cv(&self, state: &State<F>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(state.fluid.cp() - state.fluid.gas_constant())
    }
}
