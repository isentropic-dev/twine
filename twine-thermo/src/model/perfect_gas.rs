//! Calorically perfect gas model.
//!
//! `PerfectGas` implements a simple and widely-used engineering approximation:
//! an ideal gas equation of state with constant heat capacities.
//!
//! # Assumptions
//!
//! - Ideal gas equation of state: `p = ρ·R·T`
//! - Calorically perfect: `cp` and `cv` are constant (do not vary with temperature)
//!
//! # When To Use
//!
//! Use this model for quick engineering calculations when real-gas effects and
//! temperature dependence of heat capacity are negligible for your application.
//!
//! If you need temperature/pressure dependent properties or non-ideal behavior, use
//! [`super::CoolProp`] (when enabled) instead.
//!
//! # Reference State
//!
//! Enthalpy and entropy are reported relative to a configurable reference state
//! (`T_ref`, `p_ref`, `h_ref`, `s_ref`).

use std::{convert::Infallible, marker::PhantomData};

use uom::{
    ConstZero,
    si::{
        f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
        pressure::atmosphere,
        ratio::ratio,
        thermodynamic_temperature::degree_celsius,
    },
};

use thiserror::Error;

use super::ideal_gas_eos;

use crate::{
    PropertyError, State,
    capability::{
        HasCp, HasCv, HasEnthalpy, HasEntropy, HasInternalEnergy, HasPressure, StateFrom,
        ThermoModel,
    },
    units::{
        SpecificEnthalpy, SpecificEntropy, SpecificGasConstant, SpecificInternalEnergy,
        TemperatureDifference,
    },
};

#[derive(Debug, Error, Clone, PartialEq)]
pub enum PerfectGasParametersError {
    #[error("invalid gas constant R: {r:?}")]
    GasConstant { r: SpecificGasConstant },
    #[error("invalid cp: {cp:?}")]
    Cp { cp: SpecificHeatCapacity },
    #[error("invalid reference temperature: {t_ref:?}")]
    ReferenceTemperature { t_ref: ThermodynamicTemperature },
    #[error("invalid reference pressure: {p_ref:?}")]
    ReferencePressure { p_ref: Pressure },
    #[error("non-physical heat capacities: cv = cp - R must be > 0; cp={cp:?}, R={r:?}, cv={cv:?}")]
    NonPhysicalCv {
        r: SpecificGasConstant,
        cp: SpecificHeatCapacity,
        cv: SpecificHeatCapacity,
    },
}

/// Reference values used to define enthalpy/entropy offsets for a [`PerfectGas`] model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerfectGasReference {
    pub temperature: ThermodynamicTemperature,
    pub pressure: Pressure,
    pub enthalpy: SpecificEnthalpy,
    pub entropy: SpecificEntropy,
}

impl PerfectGasReference {
    /// Returns a standard reference: 0°C, 1 atm, `h_ref = 0`, `s_ref = 0`.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
            pressure: Pressure::new::<atmosphere>(1.0),
            enthalpy: SpecificEnthalpy::ZERO,
            entropy: SpecificEntropy::ZERO,
        }
    }
}

/// Constant parameters for the [`PerfectGas`] model.
///
/// These values are typically provided by a fluid's [`PerfectGasFluid`] implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerfectGasParameters {
    pub gas_constant: SpecificGasConstant,
    pub cp: SpecificHeatCapacity,
    pub reference: PerfectGasReference,
}

impl PerfectGasParameters {
    #[must_use]
    pub fn new(gas_constant: SpecificGasConstant, cp: SpecificHeatCapacity) -> Self {
        Self {
            gas_constant,
            cp,
            reference: PerfectGasReference::standard(),
        }
    }

    #[must_use]
    pub fn with_reference(mut self, reference: PerfectGasReference) -> Self {
        self.reference = reference;
        self
    }
}

/// Fluid constants required by the [`PerfectGas`] model.
pub trait PerfectGasFluid {
    /// Returns the constant parameters for use with [`PerfectGas`].
    fn parameters() -> PerfectGasParameters;
}

/// Perfect gas model (constant `cp`/`cv`) using the ideal gas equation of state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerfectGas<Fluid> {
    r: SpecificGasConstant,
    cp: SpecificHeatCapacity,
    cv: SpecificHeatCapacity,
    t_ref: ThermodynamicTemperature,
    p_ref: Pressure,
    h_ref: SpecificEnthalpy,
    s_ref: SpecificEntropy,
    _marker: PhantomData<Fluid>,
}

impl<Fluid> ThermoModel for PerfectGas<Fluid> {
    type Fluid = Fluid;
}

impl<Fluid: PerfectGasFluid> PerfectGas<Fluid> {
    /// Creates a perfect gas model using constants defined by `Fluid`.
    ///
    /// # Errors
    ///
    /// Returns [`PerfectGasParametersError`] if any required constant is
    /// invalid or if `cv = cp - R` is non-physical.
    pub fn new() -> Result<Self, PerfectGasParametersError> {
        let parameters = Fluid::parameters();

        let gas_constant = parameters.gas_constant;
        if !is_strictly_positive(gas_constant.value) {
            return Err(PerfectGasParametersError::GasConstant { r: gas_constant });
        }

        let cp = parameters.cp;
        if !is_strictly_positive(cp.value) {
            return Err(PerfectGasParametersError::Cp { cp });
        }

        let reference_temperature = parameters.reference.temperature;
        if !is_strictly_positive(reference_temperature.value) {
            return Err(PerfectGasParametersError::ReferenceTemperature {
                t_ref: reference_temperature,
            });
        }

        let reference_pressure = parameters.reference.pressure;
        if !is_strictly_positive(reference_pressure.value) {
            return Err(PerfectGasParametersError::ReferencePressure {
                p_ref: reference_pressure,
            });
        }

        let cv = cp - gas_constant;
        if !is_strictly_positive(cv.value) {
            return Err(PerfectGasParametersError::NonPhysicalCv {
                r: gas_constant,
                cp,
                cv,
            });
        }

        Ok(Self {
            r: gas_constant,
            cp,
            cv,
            t_ref: reference_temperature,
            p_ref: reference_pressure,
            h_ref: parameters.reference.enthalpy,
            s_ref: parameters.reference.entropy,
            _marker: PhantomData,
        })
    }

    /// Creates a state at the reference temperature and pressure.
    #[must_use]
    pub fn reference_state(&self, fluid: Fluid) -> State<Fluid> {
        let temperature = self.t_ref;
        let pressure = self.p_ref;
        let density = ideal_gas_eos::density(temperature, pressure, self.r);

        State {
            temperature,
            density,
            fluid,
        }
    }
}

impl<Fluid> HasPressure for PerfectGas<Fluid> {
    /// Computes pressure with `P = ρ·R·T`.
    fn pressure(&self, state: &State<Fluid>) -> Result<Pressure, PropertyError> {
        let t = state.temperature;
        let d = state.density;
        let r = self.r;

        Ok(ideal_gas_eos::pressure(t, d, r))
    }
}

impl<Fluid> HasInternalEnergy for PerfectGas<Fluid> {
    /// Computes internal energy with `u = h − R·T`.
    fn internal_energy(
        &self,
        state: &State<Fluid>,
    ) -> Result<SpecificInternalEnergy, PropertyError> {
        Ok(self.enthalpy(state)? - self.r * state.temperature)
    }
}

impl<Fluid> HasEnthalpy for PerfectGas<Fluid> {
    /// Computes enthalpy with `h = h₀ + cp·(T − T₀)`.
    fn enthalpy(&self, state: &State<Fluid>) -> Result<SpecificEnthalpy, PropertyError> {
        let cp = self.cp;
        let t_ref = self.t_ref;
        let h_ref = self.h_ref;

        Ok(h_ref + cp * state.temperature.minus(t_ref))
    }
}

impl<Fluid> HasEntropy for PerfectGas<Fluid> {
    /// Computes entropy with `s = s₀ + cp·ln(T⁄T₀) − R·ln(p⁄p₀)`.
    fn entropy(&self, state: &State<Fluid>) -> Result<SpecificEntropy, PropertyError> {
        let r = self.r;
        let cp = self.cp;
        let t_ref = self.t_ref;
        let p_ref = self.p_ref;
        let s_ref = self.s_ref;

        let p = self.pressure(state)?;

        Ok(s_ref + cp * (state.temperature / t_ref).ln() - r * (p / p_ref).ln())
    }
}

impl<Fluid> HasCp for PerfectGas<Fluid> {
    /// Returns the constant `cp` of the fluid.
    fn cp(&self, _state: &State<Fluid>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(self.cp)
    }
}

impl<Fluid> HasCv for PerfectGas<Fluid> {
    /// Returns the constant `cv` of the fluid.
    fn cv(&self, _state: &State<Fluid>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(self.cv)
    }
}

impl<Fluid> StateFrom<(Fluid, ThermodynamicTemperature, Pressure)> for PerfectGas<Fluid> {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, temperature, pressure): (Fluid, ThermodynamicTemperature, Pressure),
    ) -> Result<State<Fluid>, Self::Error> {
        let density = ideal_gas_eos::density(temperature, pressure, self.r);

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

impl<Fluid> StateFrom<(Fluid, Pressure, MassDensity)> for PerfectGas<Fluid> {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, pressure, density): (Fluid, Pressure, MassDensity),
    ) -> Result<State<Fluid>, Self::Error> {
        let temperature = ideal_gas_eos::temperature(pressure, density, self.r);

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

impl<Fluid> StateFrom<(Fluid, Pressure, SpecificEnthalpy)> for PerfectGas<Fluid> {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, pressure, enthalpy): (Fluid, Pressure, SpecificEnthalpy),
    ) -> Result<State<Fluid>, Self::Error> {
        let cp = self.cp;
        let t_ref = self.t_ref;
        let h_ref = self.h_ref;

        let temperature = t_ref + (enthalpy - h_ref) / cp;
        let density = ideal_gas_eos::density(temperature, pressure, self.r);

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

impl<Fluid> StateFrom<(Fluid, Pressure, SpecificEntropy)> for PerfectGas<Fluid> {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, pressure, entropy): (Fluid, Pressure, SpecificEntropy),
    ) -> Result<State<Fluid>, Self::Error> {
        let r = self.r;
        let cp = self.cp;
        let t_ref = self.t_ref;
        let p_ref = self.p_ref;
        let s_ref = self.s_ref;

        let exponent = ((entropy - s_ref) + r * (pressure / p_ref).ln()) / cp;
        let temperature = t_ref * exponent.get::<ratio>().exp();
        let density = ideal_gas_eos::density(temperature, pressure, r);

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

/// Returns `true` if `value` is not `NaN` and greater than zero.
fn is_strictly_positive(value: f64) -> bool {
    !value.is_nan() && value > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::pressure::kilopascal;
    use uom::si::thermodynamic_temperature::kelvin;
    use uom::si::{
        mass_density::pound_per_cubic_foot,
        pressure::{atmosphere, pascal, psi},
        specific_heat_capacity::joule_per_kilogram_kelvin,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::fluid::CarbonDioxide;

    #[derive(Debug, Clone, Copy, Default)]
    struct MockGas;

    impl PerfectGasFluid for MockGas {
        fn parameters() -> PerfectGasParameters {
            PerfectGasParameters::new(
                SpecificGasConstant::new::<joule_per_kilogram_kelvin>(400.0),
                SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1000.0),
            )
        }
    }

    fn mock_gas_model() -> PerfectGas<MockGas> {
        PerfectGas::<MockGas>::new().expect("mock gas parameters must be physically valid")
    }

    #[test]
    fn basic_properties() {
        let thermo = mock_gas_model();

        let state = thermo.reference_state(MockGas);

        let pressure_in_kpa = thermo.pressure(&state).unwrap().get::<kilopascal>();
        assert_relative_eq!(pressure_in_kpa, 101.325);

        let h_ref = thermo.enthalpy(&state).unwrap();
        assert_eq!(h_ref, SpecificEnthalpy::ZERO);
    }

    #[test]
    fn increase_temperature_at_constant_density() -> Result<(), PropertyError> {
        let thermo = mock_gas_model();

        let temp = ThermodynamicTemperature::new::<degree_celsius>(50.0);
        let pres = Pressure::new::<kilopascal>(100.0);
        let state_a: State<MockGas> = thermo.state_from((temp, pres)).unwrap();

        let state_b =
            state_a.with_temperature(ThermodynamicTemperature::new::<degree_celsius>(100.0));

        let temp_ratio = state_b.temperature / state_a.temperature;
        let expected_pressure = thermo.pressure(&state_a)? * temp_ratio;
        assert_relative_eq!(
            thermo.pressure(&state_b)?.get::<pascal>(),
            expected_pressure.get::<pascal>(),
        );

        let h_a = thermo.enthalpy(&state_a)?;
        let h_b = thermo.enthalpy(&state_b)?;
        assert!(h_b > h_a);

        Ok(())
    }

    #[test]
    fn increase_density_at_constant_temperature() -> Result<(), PropertyError> {
        let thermo = mock_gas_model();

        let pres = Pressure::new::<psi>(100.0);
        let dens = MassDensity::new::<pound_per_cubic_foot>(0.1);
        let state_a: State<MockGas> = thermo.state_from((pres, dens)).unwrap();

        let state_b = state_a.with_density(dens * 2.0);

        let expected_pressure = 2.0 * thermo.pressure(&state_a)?;
        assert_eq!(thermo.pressure(&state_b)?, expected_pressure);

        let s_a = thermo.entropy(&state_a)?;
        let s_b = thermo.entropy(&state_b)?;
        assert!(s_b < s_a);

        Ok(())
    }

    #[test]
    fn state_from_pressure_enthalpy_roundtrip() -> Result<(), PropertyError> {
        let thermo = mock_gas_model();

        let temp_in = ThermodynamicTemperature::new::<degree_celsius>(120.0);
        let pres_in = Pressure::new::<kilopascal>(250.0);
        let state_in: State<MockGas> = thermo.state_from((temp_in, pres_in)).unwrap();

        let h = thermo.enthalpy(&state_in)?;
        let state_out: State<MockGas> = thermo.state_from((pres_in, h)).unwrap();

        assert_relative_eq!(
            state_out.temperature.get::<kelvin>(),
            temp_in.get::<kelvin>(),
        );
        assert_relative_eq!(
            thermo.pressure(&state_out)?.get::<pascal>(),
            pres_in.get::<pascal>(),
        );

        Ok(())
    }

    #[test]
    fn state_from_pressure_entropy_roundtrip() -> Result<(), PropertyError> {
        let thermo = mock_gas_model();

        let temp_in = ThermodynamicTemperature::new::<degree_celsius>(80.0);
        let pres_in = Pressure::new::<kilopascal>(180.0);
        let state_in: State<MockGas> = thermo.state_from((temp_in, pres_in)).unwrap();

        let s = thermo.entropy(&state_in)?;
        let state_out: State<MockGas> = thermo.state_from((pres_in, s)).unwrap();

        assert_relative_eq!(
            state_out.temperature.get::<kelvin>(),
            temp_in.get::<kelvin>(),
        );
        assert_relative_eq!(
            thermo.pressure(&state_out)?.get::<pascal>(),
            pres_in.get::<pascal>(),
        );

        Ok(())
    }

    #[test]
    fn carbon_dioxide_parameters_smoke_test() {
        let thermo = PerfectGas::<CarbonDioxide>::new().unwrap();

        let state = thermo.reference_state(CarbonDioxide);
        assert_relative_eq!(thermo.pressure(&state).unwrap().get::<atmosphere>(), 1.0);
    }
}
