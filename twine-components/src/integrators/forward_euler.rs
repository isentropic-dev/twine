use std::{convert::Infallible, marker::PhantomData, time::Duration};

use twine_core::{Integrator, TimeDerivativeOf, TimeIntegrable};

/// A first-order explicit integrator using the forward Euler method.
///
/// This integrator is best suited for simple dynamic systems where
/// computational efficiency is more important than numerical precision.
#[derive(Debug, Default, Clone, Copy)]
pub struct ForwardEuler<T: TimeIntegrable> {
    _marker: PhantomData<T>,
}

impl<T: TimeIntegrable> ForwardEuler<T> {
    /// Creates a new `ForwardEuler` integrator.
    ///
    /// Returns a zero-sized integrator for types that implement [`TimeIntegrable`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TimeIntegrable> Integrator for ForwardEuler<T> {
    type Input = (T, TimeDerivativeOf<T>);
    type Output = T;
    type Error = Infallible;

    /// Performs a single forward Euler integration step.
    fn integrate(
        &self,
        (value, derivative): Self::Input,
        dt: Duration,
    ) -> Result<(Self::Output, Duration), Self::Error> {
        let output = value.step_by_duration(derivative, dt);
        Ok((output, dt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{Length, TemperatureInterval, ThermodynamicTemperature, Time, Velocity},
        length::meter,
        temperature_interval::degree_celsius,
        thermodynamic_temperature::kelvin,
        time::minute,
        velocity::meter_per_second,
    };

    #[test]
    fn integrates_length() {
        let integrator = ForwardEuler::<Length>::new();

        let position = Length::new::<meter>(5.0);
        let velocity = Velocity::new::<meter_per_second>(2.0);
        let dt = Duration::from_secs_f64(1.5);

        let (result, returned_dt) = integrator.integrate((position, velocity), dt).unwrap();

        assert_eq!(returned_dt, dt);
        assert_relative_eq!(result.get::<meter>(), 8.0);
    }

    #[test]
    fn integrates_temperature() {
        let integrator = ForwardEuler::<ThermodynamicTemperature>::new();

        let temperature = ThermodynamicTemperature::new::<kelvin>(300.0);
        let rate = TemperatureInterval::new::<degree_celsius>(10.0) / Time::new::<minute>(1.0);
        let dt = Duration::from_secs(30);

        let (result, returned_dt) = integrator.integrate((temperature, rate), dt).unwrap();

        assert_eq!(returned_dt, dt);
        assert_relative_eq!(result.get::<kelvin>(), 305.0);
    }
}
