use std::{ops::Div, time::Duration};

use uom::si::{f64::Time, time::second};

/// The time derivative of a quantity `T`.
///
/// This alias is useful when modeling physical systems, where `T` is a unit-aware
/// quantity from the [`uom`] crate and time is represented by [`uom::si::f64::Time`].
///
/// # Examples
///
/// - `TimeDerivativeOf<Length>` = `Velocity`
/// - `TimeDerivativeOf<Velocity>` = `Acceleration`
pub type TimeDerivativeOf<T> = <T as Div<Time>>::Output;

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
