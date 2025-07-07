use std::{
    ops::{Add, Div, Mul},
    time::Duration,
};

use uom::si::{f64::Time, time::second};

/// A trait for types that can be differentiated with respect to time.
///
/// This trait marks types that have a well-defined time derivative and support
/// applying time-based changes using standard arithmetic operations.
/// It is primarily intended for unit-aware physical quantities, such as those
/// defined using the `uom` crate.
///
/// The associated type `Derivative` represents the instantaneous rate of change.
/// The associated type `Delta` represents a finite change over a `Time` interval,
/// defined as the product of the derivative and the interval.
///
/// You do not need to implement this trait directly.
/// It is implemented automatically for any type that satisfies the following bounds:
///
/// - `T: Div<Time, Output = Derivative>`: Defines the derivative as `T / Time`.
/// - `Derivative: Mul<Time, Output = Delta>`: Defines the delta as `Derivative * Time`.
/// - `T: Add<Delta, Output = T>`: Defines how to apply a delta to the original `T`.
///
/// ### Example
///
/// To make a struct `State` compatible with `TimeDifferentiable`, implement the
/// required operations using types that represent its derivative and delta:
///
/// ```
/// use std::ops::{Add, Div, Mul};
///
/// use twine_core::TimeDerivative;
/// use uom::si::f64::*;
///
/// struct State {
///     temperature: ThermodynamicTemperature,
///     density: MassDensity,
/// }
///
/// struct StateDerivative {
///     temperature: TimeDerivative<ThermodynamicTemperature>,
///     density: TimeDerivative<MassDensity>,
/// }
///
/// struct StateDelta {
///     temperature: TemperatureInterval,
///     density: MassDensity,
/// }
///
/// impl Div<Time> for State {
///     type Output = StateDerivative;
///
///     fn div(self, rhs: Time) -> Self::Output {
///         StateDerivative {
///             temperature: self.temperature / rhs,
///             density: self.density / rhs,
///         }
///     }
/// }
///
/// impl Mul<Time> for StateDerivative {
///     type Output = StateDelta;
///
///     fn mul(self, rhs: Time) -> Self::Output {
///         StateDelta {
///             temperature: self.temperature * rhs,
///             density: self.density * rhs,
///         }
///     }
/// }
///
/// impl Add<StateDelta> for State {
///     type Output = State;
///
///     fn add(self, rhs: StateDelta) -> Self::Output {
///         State {
///             temperature: self.temperature + rhs.temperature,
///             density: self.density + rhs.density,
///         }
///     }
/// }
/// ```
///
/// With these implementations, `State` now satisfies `TimeDifferentiable`.
pub trait TimeDifferentiable
where
    Self: Div<Time, Output = Self::Derivative> + Add<Self::Delta, Output = Self>,
    Self::Derivative: Mul<Time, Output = Self::Delta>,
{
    type Derivative;
    type Delta;
}

impl<T> TimeDifferentiable for T
where
    T: Div<Time> + Add<<<T as Div<Time>>::Output as Mul<Time>>::Output, Output = T>,
    <T as Div<Time>>::Output: Mul<Time>,
{
    type Derivative = <T as Div<Time>>::Output;
    type Delta = <Self::Derivative as Mul<Time>>::Output;
}

/// The time derivative of a `TimeDifferentiable` quantity `T`.
///
/// This alias is useful when writing generic, unit-aware APIs involving
/// quantities that vary over time.
///
/// For instance, a generic integration function might be written as:
///
/// ```ignore
/// fn integrate<T: TimeDifferentiable>(value: T, rate: TimeDerivative<T>, dt: Time) -> T
/// ```
///
/// It also works naturally with `uom::Quantity` types, allowing APIs to express
/// relationships between physical quantities and their time derivatives:
///
/// - `TimeDerivative<Length>` = `Velocity`
/// - `TimeDerivative<Velocity>` = `Acceleration`
pub type TimeDerivative<T> = <T as TimeDifferentiable>::Derivative;

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
