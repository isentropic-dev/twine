use uom::si::f64::{TemperatureInterval, ThermodynamicTemperature};

use crate::controller::SwitchState;

pub struct ThermostatInput {
    pub state: SwitchState,
    pub temperature: ThermodynamicTemperature,
    pub setpoint: ThermodynamicTemperature,
    pub deadband: TemperatureInterval,
}

impl ThermostatInput {
    #[must_use]
    pub fn with_state(self, state: SwitchState) -> Self {
        Self { state, ..self }
    }

    #[must_use]
    pub fn with_temperature(self, temperature: ThermodynamicTemperature) -> Self {
        Self {
            temperature,
            ..self
        }
    }

    #[must_use]
    pub fn with_setpoint(self, setpoint: ThermodynamicTemperature) -> Self {
        Self { setpoint, ..self }
    }

    #[must_use]
    pub fn with_deadband(self, deadband: TemperatureInterval) -> Self {
        Self { deadband, ..self }
    }
}
