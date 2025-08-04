use uom::si::f64::{TemperatureInterval, ThermodynamicTemperature};

use crate::controller::SwitchState;

/// Input to a setpoint thermostat controller.
///
/// Used for both heating and cooling thermostats with hysteresis (deadband).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetpointThermostatInput {
    pub state: SwitchState,
    pub temperature: ThermodynamicTemperature,
    pub setpoint: ThermodynamicTemperature,
    pub deadband: TemperatureInterval,
}

impl SetpointThermostatInput {
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

/// A thermostat controller that maintains temperature relative to a setpoint.
///
/// Provides heating and cooling control logic with hysteresis (deadband) to prevent
/// rapid cycling between on/off states.
///
/// # Heating Mode
///
/// In heating mode, the thermostat:
/// - Turns **on** when temperature falls to `setpoint - deadband` or below.
/// - Turns **off** when temperature reaches the setpoint or above.
/// - Uses hysteresis to prevent rapid switching.
///
/// # Cooling Mode
///
/// In cooling mode, the thermostat:
/// - Turns **on** when temperature rises to `setpoint + deadband` or above.  
/// - Turns **off** when temperature reaches the setpoint or below.
/// - Uses hysteresis to prevent rapid switching.
///
/// # Examples
///
/// ```
/// use twine_components::controller::{
///     SwitchState,
///     thermostat::{SetpointThermostat, SetpointThermostatInput},
/// };
/// use uom::si::{
///     f64::{TemperatureInterval, ThermodynamicTemperature},
///     temperature_interval::degree_celsius as delta_celsius,
///     thermodynamic_temperature::degree_celsius,
/// };
///
/// let input = SetpointThermostatInput {
///     state: SwitchState::Off,
///     temperature: ThermodynamicTemperature::new::<degree_celsius>(18.0),
///     setpoint: ThermodynamicTemperature::new::<degree_celsius>(20.0),
///     deadband: TemperatureInterval::new::<delta_celsius>(2.0),
/// };
///
/// // Heating: turns on when temp <= setpoint - deadband (18°C)
/// let output = SetpointThermostat::heating(input);
/// assert_eq!(output, SwitchState::On);
///
/// // Cooling: stays off when temp < setpoint + deadband (22°C)  
/// let output = SetpointThermostat::cooling(input);
/// assert_eq!(output, SwitchState::Off);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetpointThermostat;

impl SetpointThermostat {
    /// Controls heating to maintain temperature above a setpoint.
    ///
    /// Turns the heating system on when temperature drops below `setpoint - deadband`
    /// and off when temperature reaches the setpoint.
    #[must_use]
    pub fn heating(input: SetpointThermostatInput) -> SwitchState {
        let SetpointThermostatInput {
            state,
            temperature,
            setpoint,
            deadband,
        } = input;

        match state {
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
        }
    }

    /// Controls cooling to maintain temperature below a setpoint.
    ///
    /// Turns the cooling system on when temperature rises above `setpoint + deadband`
    /// and off when temperature reaches the setpoint.
    #[must_use]
    pub fn cooling(input: SetpointThermostatInput) -> SwitchState {
        let SetpointThermostatInput {
            state,
            temperature,
            setpoint,
            deadband,
        } = input;

        match state {
            SwitchState::Off => {
                if temperature >= setpoint + deadband {
                    SwitchState::On
                } else {
                    SwitchState::Off
                }
            }
            SwitchState::On => {
                if temperature <= setpoint {
                    SwitchState::Off
                } else {
                    SwitchState::On
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        temperature_interval::degree_celsius as delta_celsius,
        thermodynamic_temperature::degree_celsius,
    };

    /// Setpoint (°C) for all tests.
    const SETPOINT: f64 = 20.0;

    /// Deadband (°C) for all tests.
    const DEADBAND: f64 = 2.0;

    fn test_input(state: SwitchState, temperature: f64) -> SetpointThermostatInput {
        SetpointThermostatInput {
            state,
            temperature: ThermodynamicTemperature::new::<degree_celsius>(temperature),
            setpoint: ThermodynamicTemperature::new::<degree_celsius>(SETPOINT),
            deadband: TemperatureInterval::new::<delta_celsius>(DEADBAND),
        }
    }

    mod heating {
        use super::*;

        #[test]
        fn turns_on_at_or_below_threshold() {
            let on_threshold = SETPOINT - DEADBAND;

            let input = test_input(SwitchState::Off, on_threshold);
            let output = SetpointThermostat::heating(input);
            assert_eq!(output, SwitchState::On);

            let input = test_input(SwitchState::Off, on_threshold - 0.1);
            let output = SetpointThermostat::heating(input);
            assert_eq!(output, SwitchState::On);
        }

        #[test]
        fn stays_on_below_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT - 0.1);
            let output = SetpointThermostat::heating(input);
            assert_eq!(output, SwitchState::On);
        }

        #[test]
        fn turns_off_at_or_above_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT);
            let output = SetpointThermostat::heating(input);
            assert_eq!(output, SwitchState::Off);

            let input = test_input(SwitchState::On, SETPOINT + 0.1);
            let output = SetpointThermostat::heating(input);
            assert_eq!(output, SwitchState::Off);
        }

        #[test]
        fn stays_off_in_deadband() {
            let on_threshold = SETPOINT - DEADBAND;
            let midpoint = f64::midpoint(SETPOINT, on_threshold);

            let input = test_input(SwitchState::Off, midpoint);
            let output = SetpointThermostat::heating(input);
            assert_eq!(output, SwitchState::Off);
        }
    }

    mod cooling {
        use super::*;

        #[test]
        fn turns_on_at_or_above_threshold() {
            let on_threshold = SETPOINT + DEADBAND;

            let input = test_input(SwitchState::Off, on_threshold);
            let output = SetpointThermostat::cooling(input);
            assert_eq!(output, SwitchState::On);

            let input = test_input(SwitchState::Off, on_threshold + 0.1);
            let output = SetpointThermostat::cooling(input);
            assert_eq!(output, SwitchState::On);
        }

        #[test]
        fn stays_on_above_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT + 0.1);
            let output = SetpointThermostat::cooling(input);
            assert_eq!(output, SwitchState::On);
        }

        #[test]
        fn turns_off_at_or_below_setpoint() {
            let input = test_input(SwitchState::On, SETPOINT);
            let output = SetpointThermostat::cooling(input);
            assert_eq!(output, SwitchState::Off);

            let input = test_input(SwitchState::On, SETPOINT - 0.1);
            let output = SetpointThermostat::cooling(input);
            assert_eq!(output, SwitchState::Off);
        }

        #[test]
        fn stays_off_in_deadband() {
            let on_threshold = SETPOINT + DEADBAND;
            let midpoint = f64::midpoint(SETPOINT, on_threshold);

            let input = test_input(SwitchState::Off, midpoint);
            let output = SetpointThermostat::cooling(input);
            assert_eq!(output, SwitchState::Off);
        }
    }
}
