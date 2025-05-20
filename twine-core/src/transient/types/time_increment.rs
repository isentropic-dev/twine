use std::{
    fmt,
    ops::{Add, AddAssign, Deref, Div},
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

    /// Returns the number of steps of size `dt` required to cover this increment.
    ///
    /// The step count is always rounded up to ensure the total interval is
    /// covered, even if `dt` does not evenly divide it.
    ///
    /// # Panics
    ///
    /// Panics if the required number of steps would exceed `usize::MAX`.
    #[must_use]
    pub fn steps_required(self, dt: TimeIncrement) -> usize {
        let duration = self.into_inner();
        let steps_f64 = (duration / *dt).value.ceil();

        #[allow(clippy::cast_precision_loss)]
        let max_steps_f64 = usize::MAX as f64;
        assert!(
            steps_f64 <= max_steps_f64,
            "Too many steps requested: {steps_f64} exceeds usize::MAX"
        );

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let steps = steps_f64 as usize;

        steps
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

/// Divides a [`TimeIncrement`] by a positive integer.
///
/// This operation is useful when subdividing an existing time increment into
/// equal, smaller intervals.
///
/// # Panics
///
/// Panics if `rhs` is zero.
impl Div<usize> for TimeIncrement {
    type Output = TimeIncrement;

    fn div(self, rhs: usize) -> Self::Output {
        assert!(rhs > 0, "Cannot divide a TimeIncrement by zero steps");

        #[allow(clippy::cast_precision_loss)]
        let dt = self.into_inner() / rhs as f64;

        TimeIncrement(dt)
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

    use uom::si::time::{minute, second};

    #[test]
    fn add_time_increments() {
        let a = TimeIncrement::new::<second>(1.0).unwrap();
        let b = TimeIncrement::new::<minute>(2.5).unwrap();
        let sum = a + b;
        assert_eq!(sum, TimeIncrement::new::<second>(151.0).unwrap());
    }

    #[test]
    fn add_time_increment_to_a_time() {
        let t = Time::new::<second>(5.0);
        let dt = TimeIncrement::new::<second>(2.0).unwrap();
        let new_time = t + dt;
        assert_eq!(new_time, Time::new::<second>(7.0));
    }

    #[test]
    fn zero_time_increment_fails() {
        assert!(TimeIncrement::new::<minute>(0.0).is_err());
    }

    #[test]
    fn negative_time_increment_fails() {
        assert!(TimeIncrement::new::<minute>(-1.0).is_err());
    }

    #[test]
    fn divide_time_increment_by_integer() {
        let total = TimeIncrement::new::<second>(10.0).unwrap();
        let dt = total / 5;
        assert_eq!(dt, TimeIncrement::new::<second>(2.0).unwrap());

        let total = TimeIncrement::new::<second>(9.0).unwrap();
        let dt = total / 4;
        assert_eq!(dt, TimeIncrement::new::<second>(2.25).unwrap());

        let total = TimeIncrement::new::<second>(5.0).unwrap();
        let dt = total / 1;
        assert_eq!(dt, total);
    }

    #[test]
    #[should_panic(expected = "Cannot divide a TimeIncrement by zero steps")]
    fn divide_time_increment_by_zero_fails() {
        let dt = TimeIncrement::new::<second>(10.0).unwrap();
        let _ = dt / 0;
    }

    #[test]
    fn steps_required_works() {
        let total = TimeIncrement::new::<second>(10.0).unwrap();

        // 10 seconds divided by 2.5 yields 4 steps (4 × 2.5 = 10.0).
        let dt = TimeIncrement::new::<second>(2.5).unwrap();
        assert_eq!(total.steps_required(dt), 4);

        // 10 seconds divided by 3 yields 4 steps (4 × 3 = 12.0, which covers the interval).
        let dt = TimeIncrement::new::<second>(3.0).unwrap();
        assert_eq!(total.steps_required(dt), 4);

        // 10 seconds divided by 1 yields 10 steps (exact division).
        let dt = TimeIncrement::new::<second>(1.0).unwrap();
        assert_eq!(total.steps_required(dt), 10);

        // If dt is almost but not quite an even divisor, the step count is rounded up.
        let dt = TimeIncrement::new::<second>(3.3).unwrap();
        assert_eq!(total.steps_required(dt), 4);

        // If dt is greater than the total duration, only 1 step is required.
        let dt = TimeIncrement::new::<second>(20.0).unwrap();
        assert_eq!(total.steps_required(dt), 1);
    }
}
