use std::{
    fmt::Debug,
    ops::{Add, Div, Mul},
    time::Duration,
};

use uom::si::{f64::Time, time::second};

/// A trait for time-differentiable types that support finite time stepping.
///
/// This trait represents types that have a well-defined time derivative and
/// support integration over a time step using standard arithmetic operations.
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
/// - `T: Div<Time, Output = Derivative>`: Defines the derivative as `T / Time`
/// - `Derivative: Mul<Time, Output = Delta>`: Defines the delta as `Derivative * Time`
/// - `T: Add<Delta, Output = T>`: Enables applying a delta to produce an updated value
/// - `T`, `Derivative`, and `Delta` implement `Debug`, `Clone`, and `PartialEq`
///
/// # Example
///
/// To make a struct `MyState` compatible with `TimeDifferentiable`, provide
/// implementations of the required operations for it and the associated
/// `Derivative` and `Delta` types:
///
/// ```
/// use std::ops::{Add, Div, Mul};
///
/// use twine_core::TimeDerivative;
/// use uom::si::f64::{MassDensity, TemperatureInterval, ThermodynamicTemperature, Time};
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyState {
///     temperature: ThermodynamicTemperature,
///     density: MassDensity,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyStateDerivative {
///     temperature: TimeDerivative<ThermodynamicTemperature>,
///     density: TimeDerivative<MassDensity>,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct MyStateDelta {
///     temperature: TemperatureInterval,
///     density: MassDensity,
/// }
///
/// impl Div<Time> for MyState {
///     type Output = MyStateDerivative;
///
///     fn div(self, rhs: Time) -> Self::Output {
///         Self::Output {
///             temperature: self.temperature / rhs,
///             density: self.density / rhs,
///         }
///     }
/// }
///
/// impl Mul<Time> for MyStateDerivative {
///     type Output = MyStateDelta;
///
///     fn mul(self, rhs: Time) -> Self::Output {
///         Self::Output {
///             temperature: self.temperature * rhs,
///             density: self.density * rhs,
///         }
///     }
/// }
///
/// impl Add<MyStateDelta> for MyState {
///     type Output = Self;
///
///     fn add(self, rhs: MyStateDelta) -> Self::Output {
///         Self::Output {
///             temperature: self.temperature + rhs.temperature,
///             density: self.density + rhs.density,
///         }
///     }
/// }
/// ```
///
/// With these implementations, `MyState` now satisfies `TimeDifferentiable`.
pub trait TimeDifferentiable:
    Debug + Clone + PartialEq + Div<Time, Output = Self::Derivative> + Add<Self::Delta, Output = Self>
{
    type Derivative: Debug + Clone + PartialEq + Mul<Time, Output = Self::Delta>;
    type Delta: Debug + Clone + PartialEq;
}

impl<T, Derivative, Delta> TimeDifferentiable for T
where
    T: Debug + Clone + PartialEq + Div<Time, Output = Derivative> + Add<Delta, Output = T>,
    Derivative: Debug + Clone + PartialEq + Mul<Time, Output = Delta>,
    Delta: Debug + Clone + PartialEq,
{
    type Derivative = Derivative;
    type Delta = Delta;
}

/// The time derivative of a `TimeDifferentiable` quantity `T`.
///
/// This alias is useful in type-level contexts (e.g., struct fields that
/// represent time derivatives), especially when working with unit-aware types
/// from the `uom` crate.
///
/// # Examples
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
