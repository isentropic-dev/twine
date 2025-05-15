use std::{
    fmt,
    ops::{Add, AddAssign, Deref},
};

use thiserror::Error;
use uom::{
    si::{f64::Time, time},
    Conversion,
};

/// A unit-safe, strictly positive duration used to advance simulation time.
///
/// `TimeIncrement` represents a discrete time step in a simulation.
/// It wraps a [`Time`] value while enforcing that the duration is strictly
/// greater than zero.
///
/// # Construction
///
/// You can create a `TimeIncrement` using either a concrete [`uom`] unit:
///
/// ```ignore
/// use twine_core::transient::TimeIncrement;
/// use uom::si::time::second;
///
/// let dt = TimeIncrement::new::<second>(1.0)?;
/// ```
///
/// Or from an existing [`Time`] value:
///
/// ```ignore
/// use twine_core::transient::TimeIncrement;
/// use uom::si::{f64::Time, time::minute};
///
/// let t = Time::new::<minute>(5.0);
/// let dt = TimeIncrement::try_from(t)?;
/// ```
///
/// # Enforcement
///
/// - Time increments must be strictly positive.
/// - Zero or negative values result in a [`TimeIncrementError::NotPositive`] error.
/// - All arithmetic preserves unit safety via [`uom`] types.
///
/// # Supported Operations
///
/// `TimeIncrement` implements the following traits:
///
/// - [`TryFrom<Time>`] — fallible construction from a raw [`Time`] value.
/// - [`Deref`] — allows transparent access to inner `Time` methods.
/// - [`Add<TimeIncrement>` for `TimeIncrement`] — accumulate multiple time steps.
/// - [`AddAssign<TimeIncrement>` for `TimeIncrement`] — in-place accumulation.
/// - [`Add<TimeIncrement>` for `Time`] — advance a `Time` by a time step.
/// - [`Display`] — renders the increment in seconds (e.g., `"60.0 s"`).
///
/// These traits offer ergonomic, type-safe operations for managing simulation
/// time and time increments.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimeIncrement(Time);

/// Error type returned when constructing an invalid [`TimeIncrement`].
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum TimeIncrementError {
    #[error("Time increment must be greater than zero, got {0} s")]
    NotPositive(f64),
}

impl TimeIncrement {
    /// Constructs a `TimeIncrement` from a numeric value and unit.
    ///
    /// Returns `Ok(Self)` if the value is strictly positive.
    ///
    /// # Parameters
    ///
    /// - `value`: The magnitude of the increment.
    /// - `U`: A unit from [`uom::si::time`] (e.g., [`second`], [`minute`]).
    ///
    /// # Errors
    ///
    /// Returns [`TimeIncrementError::NotPositive`] if `value` is zero or negative.
    pub fn new<U>(value: f64) -> Result<Self, TimeIncrementError>
    where
        U: time::Unit + Conversion<f64, T = f64>,
    {
        let t = Time::new::<U>(value);
        Self::from_time(t)
    }

    /// Constructs a `TimeIncrement` from an existing [`Time`] value.
    ///
    /// Returns `Ok(Self)` if the time is strictly positive.
    ///
    /// # Errors
    ///
    /// Returns [`TimeIncrementError::NotPositive`] if the time is zero or negative.
    pub fn from_time(time: Time) -> Result<Self, TimeIncrementError> {
        let seconds = time.get::<time::second>();
        if seconds > 0.0 {
            Ok(Self(time))
        } else {
            Err(TimeIncrementError::NotPositive(seconds))
        }
    }

    /// Consumes the `TimeIncrement` and returns the underlying [`Time`] value.
    #[must_use]
    pub fn into_inner(self) -> Time {
        self.0
    }
}

/// Attempts to convert a [`Time`] value into a [`TimeIncrement`].
///
/// Fails if the time value is zero or negative, enforcing that time increments
/// must be strictly positive.
impl TryFrom<Time> for TimeIncrement {
    type Error = TimeIncrementError;
    fn try_from(t: Time) -> Result<Self, Self::Error> {
        Self::from_time(t)
    }
}

/// Dereferences to the inner [`Time`] value.
///
/// This allows `TimeIncrement` to be used wherever a `Time` reference is
/// expected, while preserving type safety for time-stepping operations.
impl Deref for TimeIncrement {
    type Target = Time;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Adds two `TimeIncrement`s together, returning a new increment.
///
/// This operation is useful when accumulating multiple time steps.
impl Add<TimeIncrement> for TimeIncrement {
    type Output = TimeIncrement;
    fn add(self, rhs: TimeIncrement) -> Self::Output {
        TimeIncrement(self.0 + rhs.0)
    }
}

impl AddAssign for TimeIncrement {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// Adds a [`TimeIncrement`] to a [`Time`] value.
///
/// Returns a new `Time` that is offset forward by the given increment.
/// This operation is unit-safe and commonly used to advance simulation time.
impl Add<TimeIncrement> for Time {
    type Output = Time;
    fn add(self, rhs: TimeIncrement) -> Self::Output {
        self + rhs.0
    }
}

impl fmt::Display for TimeIncrement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.get::<time::second>();
        write!(f, "{s} s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::time::{minute, second};

    #[test]
    fn add_time_increments() {
        let a = TimeIncrement::new::<second>(1.0).unwrap();
        let b = TimeIncrement::new::<minute>(2.5).unwrap();
        let sum = a + b;
        assert_relative_eq!(sum.into_inner().get::<second>(), 151.0);
    }

    #[test]
    fn add_time_increment_to_a_time() {
        let t = Time::new::<second>(5.0);
        let dt = TimeIncrement::new::<second>(2.0).unwrap();
        let new_time = t + dt;
        assert_relative_eq!(new_time.get::<second>(), 7.0);
    }

    #[test]
    fn zero_time_increment_fails() {
        assert!(TimeIncrement::new::<minute>(0.0).is_err());
    }

    #[test]
    fn negative_time_increment_fails() {
        assert!(TimeIncrement::new::<minute>(-1.0).is_err());
    }
}
