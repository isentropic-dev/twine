use std::{
    fmt::Debug,
    ops::{Add, Div, Mul},
    time::Duration,
};

use uom::si::{f64::Time, time::second};

/// A trait for types that support finite time stepping via integration.
///
/// Types that implement this trait must have a well-defined time derivative and
/// be able to advance their state over a specified time interval.
///
/// This trait is primarily intended for unit-aware physical quantities,
/// such as those provided by the [`uom`] crate.
///
/// It imposes no operator bounds directly. However, it is automatically
/// implemented for any type that satisfies the following bounds:
///
/// - `Self: Div<Time, Output = Derivative>`
/// - `Derivative: Mul<Time, Output = Delta>`
/// - `Self: Add<Delta, Output = Self>`
///
/// For such types, which include all `uom` quantities,
/// integration is performed using a forward Euler step:
///
/// ```text
/// next_value = self + derivative * dt
/// ```
///
/// # Example
///
/// To implement [`TimeIntegrable`] manually for a composite type,
/// define a corresponding derivative type and delegate the integration
/// logic to each field:
///
/// ```
/// use twine_core::{TimeIntegrable, TimeDerivative};
/// use uom::si::f64::{MassDensity, ThermodynamicTemperature, Time};
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct State<T: TimeIntegrable> {
///     temperature: ThermodynamicTemperature,
///     density: MassDensity,
///     other: T,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct StateDerivative<T: TimeIntegrable> {
///     temperature: TimeDerivative<ThermodynamicTemperature>,
///     density: TimeDerivative<MassDensity>,
///     other: TimeDerivative<T>,
/// }
///
/// impl<T: TimeIntegrable> TimeIntegrable for State<T> {
///     type Derivative = StateDerivative<T>;
///
///     fn step(self, derivative: Self::Derivative, dt: Time) -> Self {
///         Self {
///             temperature: self.temperature.step(derivative.temperature, dt),
///             density: self.density.step(derivative.density, dt),
///             other: self.other.step(derivative.other, dt),
///         }
///     }
/// }
/// ```
///
/// Alternatively, you can derive this implementation automatically using the
/// `#[derive(TimeIntegrable)]` macro from the [`twine_macros`] crate:
///
/// ```ignore
/// use twine_macros::TimeIntegrable;
/// use uom::si::f64::{MassDensity, ThermodynamicTemperature};
///
/// #[derive(Debug, Clone, PartialEq, TimeIntegrable)]
/// struct State {
///     temperature: ThermodynamicTemperature,
///     density: MassDensity,
/// }
/// ```
///
/// This generates the same `StateDerivative` struct and [`TimeIntegrable`]
/// implementation as shown above.
pub trait TimeIntegrable: Debug + Clone + PartialEq {
    type Derivative: Debug + Clone + PartialEq;

    /// Advances the value using its derivative over a time interval.
    #[must_use]
    fn step(self, derivative: Self::Derivative, dt: Time) -> Self;
}

impl<T, Derivative, Delta> TimeIntegrable for T
where
    T: Debug + Clone + PartialEq,
    T: Div<Time, Output = Derivative> + Add<Delta, Output = T>,
    Derivative: Debug + Clone + PartialEq,
    Derivative: Mul<Time, Output = Delta>,
{
    type Derivative = Derivative;

    /// Computes a forward Euler integration step.
    fn step(self, derivative: Self::Derivative, dt: Time) -> Self {
        self + derivative * dt
    }
}

/// The time derivative associated with a `TimeIntegrable` type `T`.
///
/// This alias is useful in type-level contexts (e.g., struct fields that
/// represent time derivatives), especially when working with unit-aware types
/// from the `uom` crate.
///
/// # Examples
///
/// - `TimeDerivative<Length>` = `Velocity`
/// - `TimeDerivative<Velocity>` = `Acceleration`
pub type TimeDerivative<T> = <T as TimeIntegrable>::Derivative;

/// Extension trait for ergonomic operations on [`Duration`].
///
/// This trait provides additional utilities for working with [`std::time::Duration`],
/// such as unit-aware conversions and other common operations involving time.
///
/// While it currently defines only a single method, it is expected to grow into
/// a collection of [`Duration`]-related functionality as the need arises.
///
/// # Example
///
/// ```
/// use std::time::Duration;
///
/// use twine_core::DurationExt;
/// use uom::si::{f64::Time, time::second};
///
/// let dt = Duration::from_secs_f64(2.5);
/// let t: Time = dt.as_time();
///
/// assert_eq!(t.get::<second>(), 2.5);
/// ```
pub trait DurationExt {
    /// Converts this [`Duration`] into a [`uom::si::f64::Time`] quantity.
    fn as_time(&self) -> Time;
}

impl DurationExt for Duration {
    fn as_time(&self) -> Time {
        Time::new::<second>(self.as_secs_f64())
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
    fn step_length_forward() {
        let position = Length::new::<meter>(5.0);
        let velocity = Velocity::new::<meter_per_second>(2.0);
        let dt = Time::new::<second>(1.5);

        let next_position = position.step(velocity, dt);
        assert_relative_eq!(next_position.get::<meter>(), 8.0);
    }

    #[test]
    fn step_temperature_forward() {
        let temperature = ThermodynamicTemperature::new::<kelvin>(300.0);
        let rate = TemperatureInterval::new::<degree_celsius>(10.0) / Time::new::<minute>(1.0);
        let dt = Time::new::<second>(30.0);

        let next_temperature = temperature.step(rate, dt);
        assert_relative_eq!(next_temperature.get::<kelvin>(), 305.0);
    }
}
