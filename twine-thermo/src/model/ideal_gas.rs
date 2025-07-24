use std::convert::Infallible;

use uom::{
    ConstZero,
    si::{
        f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
        temperature_interval, thermodynamic_temperature,
    },
};

use crate::{
    PropertyError, State,
    fluid::Stateless,
    units::{
        SpecificEnthalpy, SpecificEntropy, SpecificGasConstant, SpecificInternalEnergy,
        TemperatureDifference,
    },
};

use super::{StateFrom, ThermodynamicProperties};

/// Trait used to define thermodynamic constants for ideal gases.
///
/// This trait provides the fixed properties required to model a fluid using
/// ideal gas assumptions, such as the specific gas constant `R`, constant
/// pressure heat capacity `cp`, and reference conditions.
///
/// Typically implemented for simple fluids like [`Air`] or [`CarbonDioxide`],
/// this trait enables reuse across models that support ideal gases,
/// such as the [`IdealGas`] model.
///
/// You can also implement this trait for any custom fluid that can be modeled
/// as an ideal gas:
///
/// ```ignore
/// use twine_thermo::{IdealGasFluid, units::SpecificGasConstant};
/// use uom::si::f64::{Pressure, SpecificHeatCapacity, ThermodynamicTemperature};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// struct MyGas;
///
/// impl IdealGasFluid for MyGas {
///     fn gas_constant(&self) -> SpecificGasConstant { /* ... */ }
///     fn cp(&self) -> SpecificHeatCapacity { /* ... */ }
///     fn reference_temperature(&self) -> ThermodynamicTemperature { /* ... */ }
///     fn reference_pressure(&self) -> Pressure { /* ... */ }
/// }
/// ```
pub trait IdealGasFluid {
    /// Returns the specific gas constant `R`.
    fn gas_constant(&self) -> SpecificGasConstant;

    /// Returns the specific heat capacity at constant pressure `cp`.
    fn cp(&self) -> SpecificHeatCapacity;

    /// Returns the reference temperature used in enthalpy and entropy calculations.
    fn reference_temperature(&self) -> ThermodynamicTemperature;

    /// Returns the reference pressure used in entropy calculations.
    fn reference_pressure(&self) -> Pressure;

    /// Returns the enthalpy at the reference temperature.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_enthalpy(&self) -> SpecificEnthalpy {
        SpecificEnthalpy::ZERO
    }

    /// Returns the entropy at the reference temperature and pressure.
    ///
    /// Defaults to zero.
    /// Override to use a nonzero reference value.
    fn reference_entropy(&self) -> SpecificEntropy {
        SpecificEntropy::ZERO
    }
}

/// A fluid property model using ideal gas assumptions.
///
/// Assumes ideal gas behavior and constant specific heat, making it
/// suitable for conditions where real gas effects are negligible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IdealGas;

impl IdealGas {
    /// Creates a state at the fluid's reference temperature and pressure.
    #[must_use]
    pub fn reference_state<F: IdealGasFluid>(fluid: F) -> State<F> {
        let temperature = fluid.reference_temperature();
        let pressure = fluid.reference_pressure();
        let density = IdealGas::density(temperature, pressure, fluid.gas_constant());

        State {
            temperature,
            density,
            fluid,
        }
    }

    /// Computes pressure using the ideal gas law.
    #[must_use]
    pub fn pressure(
        temperature: ThermodynamicTemperature,
        density: MassDensity,
        gas_constant: SpecificGasConstant,
    ) -> Pressure {
        density * gas_constant * temperature
    }

    /// Computes density using the ideal gas law.
    #[must_use]
    pub fn density(
        temperature: ThermodynamicTemperature,
        pressure: Pressure,
        gas_constant: SpecificGasConstant,
    ) -> MassDensity {
        pressure / (gas_constant * temperature)
    }

    /// Computes temperature using the ideal gas law.
    ///
    /// Since `SpecificGasConstant` is associated with a `TemperatureInterval`,
    /// the result must be manually converted to an absolute temperature.
    /// This conversion is safe because the ideal gas law naturally produces
    /// absolute temperature values.
    #[must_use]
    pub fn temperature(
        pressure: Pressure,
        density: MassDensity,
        gas_constant: SpecificGasConstant,
    ) -> ThermodynamicTemperature {
        let temperature = pressure / (density * gas_constant);
        ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(
            temperature.get::<temperature_interval::kelvin>(),
        )
    }
}

impl<F: IdealGasFluid> ThermodynamicProperties<F> for IdealGas {
    /// Computes pressure with `P = ρ·R·T`.
    fn pressure(&self, state: &State<F>) -> Result<Pressure, PropertyError> {
        let t = state.temperature;
        let d = state.density;
        let r = state.fluid.gas_constant();

        Ok(IdealGas::pressure(t, d, r))
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

/// Enables state creation from temperature and pressure for any [`Stateless`] fluid.
impl<F: IdealGasFluid + Stateless> StateFrom<F, (ThermodynamicTemperature, Pressure)> for IdealGas {
    type Error = Infallible;

    fn state_from(
        &self,
        (temperature, pressure): (ThermodynamicTemperature, Pressure),
    ) -> Result<State<F>, Self::Error> {
        let fluid = F::default();
        let density = IdealGas::density(temperature, pressure, fluid.gas_constant());

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

/// Enables state creation from pressure and density for any [`Stateless`] fluid.
impl<F: IdealGasFluid + Stateless> StateFrom<F, (Pressure, MassDensity)> for IdealGas {
    type Error = Infallible;

    fn state_from(
        &self,
        (pressure, density): (Pressure, MassDensity),
    ) -> Result<State<F>, Self::Error> {
        let fluid = F::default();
        let temperature = IdealGas::temperature(pressure, density, fluid.gas_constant());

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
        mass_density::pound_per_cubic_foot,
        pressure::{atmosphere, kilopascal, pascal, psi},
        specific_heat_capacity::joule_per_kilogram_kelvin,
        thermodynamic_temperature::degree_celsius,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct MockGas;

    impl Stateless for MockGas {}

    impl IdealGasFluid for MockGas {
        fn gas_constant(&self) -> SpecificGasConstant {
            SpecificGasConstant::new::<joule_per_kilogram_kelvin>(400.0)
        }

        fn cp(&self) -> SpecificHeatCapacity {
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1000.0)
        }

        fn reference_temperature(&self) -> ThermodynamicTemperature {
            ThermodynamicTemperature::new::<degree_celsius>(0.0)
        }

        fn reference_pressure(&self) -> Pressure {
            Pressure::new::<atmosphere>(1.0)
        }
    }

    #[test]
    fn basic_properties() {
        // State at reference temperature and density.
        let state = IdealGas::reference_state(MockGas);

        let pressure_in_kpa = IdealGas.pressure(&state).unwrap().get::<kilopascal>();
        assert_relative_eq!(pressure_in_kpa, 101.325);

        let h_ref = IdealGas.enthalpy(&state).unwrap();
        assert_eq!(h_ref, SpecificEnthalpy::ZERO);
    }

    #[test]
    fn increase_temperature_at_constant_density() -> Result<(), PropertyError> {
        // State from a temperature and pressure.
        let temp = ThermodynamicTemperature::new::<degree_celsius>(50.0);
        let pres = Pressure::new::<kilopascal>(100.0);
        let state_a: State<MockGas> = IdealGas.state_from((temp, pres)).unwrap();

        let state_b =
            state_a.with_temperature(ThermodynamicTemperature::new::<degree_celsius>(100.0));

        // Check that pressure increased as expected based on the temperature ratio.
        let temp_ratio = state_b.temperature / state_a.temperature;
        let expected_pressure = IdealGas.pressure(&state_a)? * temp_ratio;
        assert_relative_eq!(
            IdealGas.pressure(&state_b)?.get::<pascal>(),
            expected_pressure.get::<pascal>(),
        );

        // Check that enthalpy increases with temperature.
        let h_a = IdealGas.enthalpy(&state_a)?;
        let h_b = IdealGas.enthalpy(&state_b)?;
        assert!(h_b > h_a);

        Ok(())
    }

    #[test]
    fn increase_density_at_constant_temperature() -> Result<(), PropertyError> {
        // State from a pressure and density.
        let pres = Pressure::new::<psi>(100.0);
        let dens = MassDensity::new::<pound_per_cubic_foot>(0.1);
        let state_a: State<MockGas> = IdealGas.state_from((pres, dens)).unwrap();

        let state_b = state_a.with_density(dens * 2.0);

        // Check that pressure doubled as expected based on the density ratio.
        let expected_pressure = 2.0 * IdealGas.pressure(&state_a)?;
        assert_eq!(IdealGas.pressure(&state_b)?, expected_pressure);

        // Check that entropy decreases with density.
        let s_a = IdealGas.entropy(&state_a)?;
        let s_b = IdealGas.entropy(&state_b)?;
        assert!(s_b < s_a);

        Ok(())
    }
}
