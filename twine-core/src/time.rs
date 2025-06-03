use std::{
    ops::{Add, Div, Mul},
    time::Duration,
};
use uom::si::{f64::Time, time::second};

/// The time derivative of a quantity `T`.
///
/// This alias is commonly used when modeling physical systems, where `T` is a
/// quantity from the [`uom`] crate and `Time` is [`uom::si::f64::Time`].
///
/// # Examples
///
/// - `TimeDerivativeOf<Length>` = `Velocity`
/// - `TimeDerivativeOf<Velocity>` = `Acceleration`
pub type TimeDerivativeOf<T> = <T as Div<Time>>::Output;

/// Trait for types that support numeric integration over time.
///
/// Describes how to step a value through time using its derivative.
/// Commonly used in simulations and physical models where quantities evolve
/// according to their rate of change.
///
/// # Methods
///
/// - [`step_by_time`] (required): steps the value using a `uom::Time`.
/// - [`step_by_duration`] (provided): steps the value using a `std::time::Duration`.
pub trait TimeIntegrable: Sized + Div<Time> {
    /// Steps the value by a time increment `dt`.
    #[must_use]
    fn step_by_time(self, derivative: TimeDerivativeOf<Self>, dt: Time) -> Self;

    /// Steps the value by a standard `Duration`.
    ///
    /// Converts the duration to a `uom::Time` and calls `step_by_time`.
    #[must_use]
    fn step_by_duration(self, derivative: TimeDerivativeOf<Self>, dt: Duration) -> Self {
        let dt = Time::new::<second>(dt.as_secs_f64());
        self.step_by_time(derivative, dt)
    }
}

/// Blanket implementation of [`TimeIntegrable`] using the explicit Euler method.
///
/// Applies to any type that supports division by time, multiplication of its
/// derivative by time, and addition of the result back to itself:
///
/// ```text
/// next = self + derivative * dt
/// ```
impl<T> TimeIntegrable for T
where
    T: Div<Time>,
    TimeDerivativeOf<T>: Mul<Time>,
    T: Add<<TimeDerivativeOf<T> as Mul<Time>>::Output, Output = T>,
{
    fn step_by_time(self, derivative: TimeDerivativeOf<Self>, dt: Time) -> Self {
        self + derivative * dt
    }
}
