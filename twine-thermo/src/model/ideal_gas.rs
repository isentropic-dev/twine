use std::convert::Infallible;

use twine_core::{
    TimeIntegrable,
    constraint::{Constrained, NonNegative, StrictlyPositive},
};
use uom::{
    ConstZero,
    si::{
        f64::{
            MassDensity, MassRate, Power, Pressure, SpecificHeatCapacity, ThermodynamicTemperature,
            Volume,
        },
        temperature_interval, thermodynamic_temperature,
    },
};

use crate::{
    Flow, PropertyError, State, StateDerivative,
    fluid::Stateless,
    units::{
        SpecificEnthalpy, SpecificEntropy, SpecificGasConstant, SpecificInternalEnergy,
        TemperatureDifference,
    },
};

use super::{ControlVolumeFixedFlow, StateFrom, ThermodynamicProperties};

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

/// Implements [`ControlVolumeFixedFlow`] for any [`IdealGasFluid`] with no time-dependent state.
///
/// A fluid has no time-dependent state if its `TimeIntegrable::Derivative` type is `()`.
impl<F> ControlVolumeFixedFlow<F> for IdealGas
where
    F: IdealGasFluid + TimeIntegrable<Derivative = ()>,
{
    fn state_derivative(
        &self,
        volume: Constrained<Volume, StrictlyPositive>,
        state: &State<F>,
        inflows: &[Flow<F>],
        outflow: Constrained<MassRate, NonNegative>,
        heat_input: Power,
        power_output: Power,
    ) -> Result<StateDerivative<F>, PropertyError> {
        let vol = volume.into_inner();
        let mass = vol * state.density;
        let cv = self.cv(state)?;

        // Incoming flow
        let (m_dot_in, q_dot_in) = inflows.iter().fold(
            (MassRate::ZERO, Power::ZERO),
            |(m_dot_total, q_dot_total), flow| {
                let m_dot_flow = flow.mass_rate.into_inner();
                let h_flow = self.enthalpy(&flow.state).expect("Infallible for IdealGas");

                (m_dot_total + m_dot_flow, q_dot_total + m_dot_flow * h_flow)
            },
        );

        // Outgoing flow
        let m_dot_out = outflow.into_inner();
        let q_dot_out = m_dot_out * self.enthalpy(state)?;

        // Mass balance
        let dens_dt = (m_dot_in - m_dot_out) / vol;

        // Energy balance
        let q_dot_net = q_dot_in - q_dot_out + heat_input - power_output;
        let temp_dt = q_dot_net / (mass * cv);

        Ok(StateDerivative::<F> {
            temperature: temp_dt,
            density: dens_dt,
            fluid: (),
        })
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
        f64::{MassRate, Power, Time, Volume},
        mass_density::{kilogram_per_cubic_meter, pound_per_cubic_foot},
        mass_rate::kilogram_per_second,
        pressure::{atmosphere, kilopascal, pascal, psi},
        specific_heat_capacity::joule_per_kilogram_kelvin,
        thermodynamic_temperature::{degree_celsius, kelvin},
        time::second,
        volume::cubic_meter,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct MockGas;

    impl Stateless for MockGas {}

    impl TimeIntegrable for MockGas {
        type Derivative = ();

        fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
            self
        }
    }

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

    #[test]
    fn control_volume_fixed_flow_conserves_mass_and_energy() {
        let volume = Constrained::new(Volume::new::<cubic_meter>(2.0)).unwrap();

        let state = State::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            MockGas,
        );

        // Incoming flow is warmer than the state and has the same density.
        let inflow = Flow::new(
            Constrained::new(MassRate::new::<kilogram_per_second>(0.3)).unwrap(),
            state.with_temperature(ThermodynamicTemperature::new::<kelvin>(350.0)),
        );

        // Outgoing flow has the same mass rate as the incoming flow.
        let m_dot_out = inflow.mass_rate;

        // No heat or work terms.
        let heat_input = Power::ZERO;
        let power_output = Power::ZERO;

        let derivative = IdealGas
            .state_derivative(
                volume,
                &state,
                &[inflow],
                m_dot_out,
                heat_input,
                power_output,
            )
            .unwrap();

        // Density is constant when inflow and outflow rates are equal.
        assert_relative_eq!(derivative.density.value, 0.0);

        // Hand calculation for the temperature derivative:
        //   q_dot_net = m_dot * cp * (T_in - T) = 0.3 * 1,000 * (350 - 300) = 15,00
        //   c = density * volume * cv = 1 * 2 * 600 = 1,200
        //   dT/dt = q_dot_net / c = 15,000 / 1,200 = 12.5
        assert_relative_eq!(derivative.temperature.value, 12.5);
    }

    #[test]
    fn control_volume_fixed_flow_only_outflow() {
        let volume = Constrained::new(Volume::new::<cubic_meter>(2.0)).unwrap();

        let state = State::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            MockGas,
        );

        let m_dot_out = Constrained::new(MassRate::new::<kilogram_per_second>(0.2)).unwrap();

        let heat_input = Power::ZERO;
        let power_output = Power::ZERO;

        let derivative = IdealGas
            .state_derivative(volume, &state, &[], m_dot_out, heat_input, power_output)
            .unwrap();

        // Density decreases at rate of outflow over volume.
        assert_relative_eq!(derivative.density.value, -0.1);

        // Hand calculation for the temperature derivative:
        //   q_dot_out = m_dot * cp * (T - T_ref) = 0.2 * 1,000 * (300 - 273.15) = 5,370
        //   c = density * volume * cv = 1 * 2 * 600 = 1,200
        //   dT/dt = -q_dot_out / c = 5,370 / 1,200 = -4.475
        assert_relative_eq!(derivative.temperature.value, -4.475, epsilon = 1e-12);

        // Hand calculation for current pressure:
        //   P = density * R * T = 1.0 * 400 * 300 = 120,000 Pa = 120 kPa
        let current_pressure = IdealGas.pressure(&state).unwrap();
        assert_relative_eq!(current_pressure.get::<kilopascal>(), 120.0);

        // Evolve state by 1 second using the computed derivatives.
        let future_state = state.step(derivative, Time::new::<second>(1.0));
        assert_relative_eq!(future_state.density.get::<kilogram_per_cubic_meter>(), 0.9);
        assert_relative_eq!(future_state.temperature.get::<kelvin>(), 295.525);

        // Hand calculation for future pressure:
        //   P = density * R * T = 0.9 * 400 * 295.525 = 106.389 kPa
        let future_pressure = IdealGas.pressure(&future_state).unwrap();
        assert_relative_eq!(future_pressure.get::<kilopascal>(), 106.389);
    }
}
