//! Shared fixtures for turbomachinery unit tests.
//!
//! These helpers keep inline `#[cfg(test)]` modules small and ensure test cases
//! use consistent dummy fluids and controllable thermodynamic-model behaviors.

use thiserror::Error;
use twine_thermo::{
    PropertyError, State,
    fluid::Stateless,
    model::ideal_gas::IdealGasFluid,
    model::{StateFrom, ThermodynamicProperties},
    units::{SpecificEnthalpy, SpecificEntropy, SpecificGasConstant, SpecificInternalEnergy},
};
use uom::si::{
    energy::joule,
    f64::{Energy, Mass, MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
    mass::kilogram,
    mass_density::kilogram_per_cubic_meter,
    pressure::kilopascal,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::kelvin,
};

/// Ideal-gas test fluid with `k = 1.4` and a convenient reference state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MockGas;

impl Stateless for MockGas {}

impl IdealGasFluid for MockGas {
    fn gas_constant(&self) -> SpecificGasConstant {
        // Choose R so that k = cp/(cp-R) is exactly 1.4.
        SpecificGasConstant::new::<joule_per_kilogram_kelvin>(2000.0 / 7.0)
    }

    fn cp(&self) -> SpecificHeatCapacity {
        SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1000.0)
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(300.0)
    }

    fn reference_pressure(&self) -> Pressure {
        Pressure::new::<kilopascal>(100.0)
    }
}

/// Constructs a specific enthalpy in SI units (J/kg).
pub(crate) fn enth_si(value: f64) -> SpecificEnthalpy {
    Energy::new::<joule>(value) / Mass::new::<kilogram>(1.0)
}

/// Failure/behavior modes for [`FakeThermo`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FakeMode {
    /// Fail `state_from((p_out, s_in))`.
    FailStateFromPressureEntropy,
    /// Fail `state_from((p_out, h_out_target))`.
    FailStateFromPressureEnthalpy,
    /// Make `enthalpy(&state)` return an error.
    FailEnthalpy,
    /// Make `enthalpy(&state)` always return this value.
    FixedEnthalpy(SpecificEnthalpy),
}

/// Minimal thermodynamic model used to exercise error paths and wrapper behavior.
///
/// The turbomachinery core models wrap failures from state construction and/or
/// property evaluation, and treat negative work targets as non-physical.
/// `FakeThermo` provides a few controllable behaviors to test those branches
/// without depending on any specific real-fluid implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FakeThermo {
    pub(crate) mode: FakeMode,
}

#[derive(Debug, Error)]
#[error("fake state_from failure")]
pub(crate) struct FakeStateFromError;

fn fake_state() -> State<MockGas> {
    State {
        temperature: ThermodynamicTemperature::new::<kelvin>(1.0),
        density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
        fluid: MockGas,
    }
}

impl ThermodynamicProperties<MockGas> for FakeThermo {
    fn pressure(&self, _state: &State<MockGas>) -> Result<Pressure, PropertyError> {
        Err(PropertyError::NotImplemented {
            property: "pressure",
            context: None,
        })
    }

    fn internal_energy(
        &self,
        _state: &State<MockGas>,
    ) -> Result<SpecificInternalEnergy, PropertyError> {
        Err(PropertyError::NotImplemented {
            property: "internal_energy",
            context: None,
        })
    }

    fn enthalpy(&self, _state: &State<MockGas>) -> Result<SpecificEnthalpy, PropertyError> {
        match self.mode {
            FakeMode::FailEnthalpy => Err(PropertyError::Calculation("fake".into())),
            FakeMode::FixedEnthalpy(value) => Ok(value),
            _ => Ok(enth_si(1.0)),
        }
    }

    fn entropy(&self, _state: &State<MockGas>) -> Result<SpecificEntropy, PropertyError> {
        Err(PropertyError::NotImplemented {
            property: "entropy",
            context: None,
        })
    }

    fn cp(&self, _state: &State<MockGas>) -> Result<SpecificHeatCapacity, PropertyError> {
        Err(PropertyError::NotImplemented {
            property: "cp",
            context: None,
        })
    }

    fn cv(&self, _state: &State<MockGas>) -> Result<SpecificHeatCapacity, PropertyError> {
        Err(PropertyError::NotImplemented {
            property: "cv",
            context: None,
        })
    }
}

impl StateFrom<MockGas, (Pressure, SpecificEntropy)> for FakeThermo {
    type Error = FakeStateFromError;

    fn state_from(
        &self,
        _input: (Pressure, SpecificEntropy),
    ) -> Result<State<MockGas>, Self::Error> {
        match self.mode {
            FakeMode::FailStateFromPressureEntropy => Err(FakeStateFromError),
            _ => Ok(fake_state()),
        }
    }
}

impl StateFrom<MockGas, (Pressure, SpecificEnthalpy)> for FakeThermo {
    type Error = FakeStateFromError;

    fn state_from(
        &self,
        _input: (Pressure, SpecificEnthalpy),
    ) -> Result<State<MockGas>, Self::Error> {
        match self.mode {
            FakeMode::FailStateFromPressureEnthalpy => Err(FakeStateFromError),
            _ => Ok(fake_state()),
        }
    }
}
