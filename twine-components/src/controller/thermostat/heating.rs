use std::convert::Infallible;

use twine_core::Component;

use crate::controller::SwitchState;

use super::ThermostatInput;

pub struct HeatingThermostat;

impl Component for HeatingThermostat {
    type Input = ThermostatInput;
    type Output = SwitchState;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let ThermostatInput {
            state,
            temperature,
            setpoint,
            deadband,
        } = input;

        Ok(match state {
            SwitchState::On => {
                if temperature >= setpoint {
                    SwitchState::Off
                } else {
                    SwitchState::On
                }
            }
            SwitchState::Off => {
                if temperature < setpoint - deadband {
                    SwitchState::On
                } else {
                    SwitchState::Off
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::{
        f64::{TemperatureInterval, ThermodynamicTemperature},
        temperature_interval,
        thermodynamic_temperature::degree_celsius,
    };

    /// Setpoint (°C) for all tests.
    const SETPOINT: f64 = 21.0;

    /// Deadband (°C) for all tests.
    const DEADBAND: f64 = 2.0;

    fn test_input(state: SwitchState, temp_c: f64) -> ThermostatInput {
        ThermostatInput {
            state,
            temperature: ThermodynamicTemperature::new::<degree_celsius>(temp_c),
            setpoint: ThermodynamicTemperature::new::<degree_celsius>(SETPOINT),
            deadband: TemperatureInterval::new::<temperature_interval::degree_celsius>(DEADBAND),
        }
    }

    #[test]
    fn turns_on_below_on_threshold() {
        let on_threshold = SETPOINT - DEADBAND;
        let input = test_input(SwitchState::Off, on_threshold - 0.1);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::On);
    }

    #[test]
    fn stays_on_below_setpoint() {
        let input = test_input(SwitchState::On, SETPOINT - 0.1);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::On);
    }

    #[test]
    fn turns_off_at_setpoint() {
        let input = test_input(SwitchState::On, SETPOINT);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::Off);
    }

    #[test]
    fn turns_off_above_setpoint() {
        let input = test_input(SwitchState::On, SETPOINT + 1.0);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::Off);
    }

    #[test]
    fn stays_off_above_on_threshold() {
        let on_threshold = SETPOINT - DEADBAND;
        let input = test_input(SwitchState::Off, on_threshold + 0.1);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::Off);
    }

    #[test]
    fn stays_off_at_on_threshold() {
        let on_threshold = SETPOINT - DEADBAND;
        let input = test_input(SwitchState::Off, on_threshold);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::Off);
    }
}
