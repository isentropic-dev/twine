use std::convert::Infallible;

use twine_core::Component;

use crate::controller::SwitchState;

use super::ThermostatInput;

/// A heating thermostat with hysteresis (deadband).
///
/// This controller switches heating `On` at or below `setpoint - deadband`,
/// and switches it `Off` at or above the setpoint.
/// This one-sided deadband prevents rapid cycling near the setpoint.
///
/// More specifically:
/// - If currently `Off` and `temperature <= setpoint - deadband`, returns `On`.
/// - If currently `On` and `temperature >= setpoint`, returns `Off`.
/// - Otherwise, the current [`SwitchState`] is returned unchanged.
///
/// # Example
///
/// ```
///  use twine_core::Component;
///  use twine_components::controller::{
///      SwitchState,
///      thermostat::{HeatingThermostat, ThermostatInput},
///  };
///  use uom::si::{
///      f64::{TemperatureInterval, ThermodynamicTemperature},
///      temperature_interval::degree_celsius as delta_celsius,
///      thermodynamic_temperature::degree_celsius,
///  };
///
/// let input = ThermostatInput {
///     state: SwitchState::Off,
///     temperature: ThermodynamicTemperature::new::<degree_celsius>(15.0),
///     setpoint: ThermodynamicTemperature::new::<degree_celsius>(20.0),
///     deadband: TemperatureInterval::new::<delta_celsius>(2.0),
/// };
/// let output = HeatingThermostat.call(input).unwrap();
/// assert_eq!(output, SwitchState::On);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            SwitchState::Off => {
                if temperature <= setpoint - deadband {
                    SwitchState::On
                } else {
                    SwitchState::Off
                }
            }
            SwitchState::On => {
                if temperature >= setpoint {
                    SwitchState::Off
                } else {
                    SwitchState::On
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
    fn turns_on_at_or_below_on_threshold() {
        let on_threshold = SETPOINT - DEADBAND;

        let input = test_input(SwitchState::Off, on_threshold);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::On);

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
    fn turns_off_at_or_above_setpoint() {
        let input = test_input(SwitchState::On, SETPOINT);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::Off);

        let input = test_input(SwitchState::On, SETPOINT + 0.1);
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
    fn stays_off_in_deadband() {
        let on_threshold = SETPOINT - DEADBAND;
        let midpoint = f64::midpoint(on_threshold, SETPOINT);

        let input = test_input(SwitchState::Off, midpoint);
        let output = HeatingThermostat.call(input).unwrap();
        assert_eq!(output, SwitchState::Off);
    }
}
