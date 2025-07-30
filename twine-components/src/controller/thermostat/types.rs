use uom::si::f64::{TemperatureInterval, ThermodynamicTemperature};

use crate::controller::SwitchState;

/// Input to a thermostat controller.
///
/// Used for both heating and cooling thermostats with hysteresis (deadband).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermostatInput {
    pub state: SwitchState,
    pub temperature: ThermodynamicTemperature,
    pub setpoint: ThermodynamicTemperature,
    pub deadband: TemperatureInterval,
}

impl ThermostatInput {
    /// Returns `self` with the given state, keeping other fields unchanged.
    #[must_use]
    pub fn with_state(self, state: SwitchState) -> Self {
        Self { state, ..self }
    }

    /// Returns `self` with the given temperature, keeping other fields unchanged.
    #[must_use]
    pub fn with_temperature(self, temperature: ThermodynamicTemperature) -> Self {
        Self {
            temperature,
            ..self
        }
    }

    /// Returns `self` with the given setpoint, keeping other fields unchanged.
    #[must_use]
    pub fn with_setpoint(self, setpoint: ThermodynamicTemperature) -> Self {
        Self { setpoint, ..self }
    }

    /// Returns `self` with the given deadband, keeping other fields unchanged.
    #[must_use]
    pub fn with_deadband(self, deadband: TemperatureInterval) -> Self {
        Self { deadband, ..self }
    }
}
