//! A first-order explicit integrator using the forward Euler method.
//!
//! This integrator is best suited for simple dynamic systems where
//! computational efficiency takes priority over numerical accuracy.

use std::{convert::Infallible, marker::PhantomData};

use twine_core::{Component, TimeDerivative, TimeDifferentiable};
use uom::si::f64::Time;

/// Performs a forward Euler integration step: `value + derivative * dt`.
#[must_use]
pub fn step<T: TimeDifferentiable>(value: T, derivative: TimeDerivative<T>, dt: Time) -> T {
    value + derivative * dt
}

/// A [`Component`] that performs a single forward Euler integration step.
///
/// Takes a `(value, derivative, dt)` tuple and returns the integrated result.
#[derive(Debug, Clone, Copy, Default)]
pub struct ForwardEuler<T> {
    _marker: PhantomData<T>,
}

impl<T> ForwardEuler<T> {
    /// Creates a new [`ForwardEuler`] component.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TimeDifferentiable> Component for ForwardEuler<T> {
    type Input = (T, TimeDerivative<T>, Time);
    type Output = T;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (value, derivative, dt) = input;
        Ok(step(value, derivative, dt))
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
        time::{minute, second},
        velocity::meter_per_second,
    };

    #[test]
    fn integrates_length() {
        let position = Length::new::<meter>(5.0);
        let velocity = Velocity::new::<meter_per_second>(2.0);
        let dt = Time::new::<second>(1.5);

        let next_position = step(position, velocity, dt);
        assert_relative_eq!(next_position.get::<meter>(), 8.0);
    }

    #[test]
    fn integrates_temperature() {
        let temperature = ThermodynamicTemperature::new::<kelvin>(300.0);
        let rate = TemperatureInterval::new::<degree_celsius>(10.0) / Time::new::<minute>(1.0);
        let dt = Time::new::<second>(30.0);

        let next_temperature = step(temperature, rate, dt);
        assert_relative_eq!(next_temperature.get::<kelvin>(), 305.0);
    }
}
