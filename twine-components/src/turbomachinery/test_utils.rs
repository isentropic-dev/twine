//! Shared fixtures for turbomachinery unit tests.
//!
//! These helpers keep inline `#[cfg(test)]` modules small and ensure test cases
//! use consistent dummy fluids and controllable thermodynamic-model behaviors.

use thiserror::Error;
use twine_thermo::{
    PropertyError, State,
    capability::{HasEnthalpy, StateFrom, ThermoModel},
    model::perfect_gas::{PerfectGas, PerfectGasFluid, PerfectGasParameters},
    units::{SpecificEnthalpy, SpecificEntropy, SpecificGasConstant},
};
use uom::si::{
    energy::joule,
    f64::{Energy, Mass, MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
    mass::kilogram,
    mass_density::kilogram_per_cubic_meter,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::kelvin,
};

/// Perfect gas test fluid with `k = 1.4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct MockGas;

impl PerfectGasFluid for MockGas {
    fn parameters() -> PerfectGasParameters {
        // Choose R so that k = cp/(cp-R) is exactly 1.4.
        PerfectGasParameters::new(
            SpecificGasConstant::new::<joule_per_kilogram_kelvin>(2000.0 / 7.0),
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1000.0),
        )
    }
}

pub(crate) fn mock_gas_model() -> PerfectGas<MockGas> {
    PerfectGas::<MockGas>::new().expect("mock gas parameters must be physically valid")
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

impl ThermoModel for FakeThermo {
    type Fluid = MockGas;
}

impl HasEnthalpy for FakeThermo {
    fn enthalpy(&self, _state: &State<MockGas>) -> Result<SpecificEnthalpy, PropertyError> {
        match self.mode {
            FakeMode::FailEnthalpy => Err(PropertyError::Calculation {
                context: "fake".into(),
            }),
            FakeMode::FixedEnthalpy(value) => Ok(value),
            _ => Ok(enth_si(1.0)),
        }
    }
}

impl StateFrom<(MockGas, Pressure, SpecificEntropy)> for FakeThermo {
    type Error = FakeStateFromError;

    fn state_from(
        &self,
        (_fluid, _p, _s): (MockGas, Pressure, SpecificEntropy),
    ) -> Result<State<MockGas>, Self::Error> {
        match self.mode {
            FakeMode::FailStateFromPressureEntropy => Err(FakeStateFromError),
            _ => Ok(fake_state()),
        }
    }
}

impl StateFrom<(MockGas, Pressure, SpecificEnthalpy)> for FakeThermo {
    type Error = FakeStateFromError;

    fn state_from(
        &self,
        (_fluid, _p, _h): (MockGas, Pressure, SpecificEnthalpy),
    ) -> Result<State<MockGas>, Self::Error> {
        match self.mode {
            FakeMode::FailStateFromPressureEnthalpy => Err(FakeStateFromError),
            _ => Ok(fake_state()),
        }
    }
}
