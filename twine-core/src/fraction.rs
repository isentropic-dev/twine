use std::{cmp::Ordering, convert::TryFrom, ops::Mul};

use thiserror::Error;

/// A bounded scalar in `[0.0, 1.0]`.
///
/// Useful for proportions, shares, and probabilities.
///
/// This type internally wraps an `f64` and guarantees the value is within `[0, 1]`.
/// Because of this invariant, `Fraction` implements [`Eq`] and [`Ord`] even
/// though raw `f64` does not.
///
/// # Examples
/// ```
/// use twine_core::Fraction;
///
/// // Using `new`
/// let f1 = Fraction::new(0.1).unwrap();
/// assert_eq!(f1.get(), 0.1);
///
/// // Using `TryFrom<f64>`
/// let f2: Fraction = Fraction::try_from(0.2).unwrap();
/// assert_eq!(f2.get(), 0.2);
///
/// // Multiply a scalar by a fraction (either order)
/// let value = 500.0;
/// assert_eq!(f1 * value, 50.0);
/// assert_eq!(value * f1, 50.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fraction(f64);

impl Fraction {
    /// Creates a `Fraction` if `value` is within `[0, 1]`.
    ///
    /// # Errors
    ///
    /// Returns [`FractionError::NotFinite`] if `value` is `NaN` or infinite.
    /// Returns [`FractionError::OutOfRange`] if `value` is less than `0.0`
    /// or greater than `1.0`.
    pub fn new(value: f64) -> Result<Self, FractionError> {
        if !value.is_finite() {
            return Err(FractionError::NotFinite(value));
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(FractionError::OutOfRange(value));
        }
        Ok(Self(value))
    }

    /// Creates a `Fraction` from a percentage within `[0, 100]`.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Fraction::new`] if the derived value is not
    /// finite or lies outside `[0, 1]`.
    pub fn from_percent(percent: f64) -> Result<Self, FractionError> {
        Self::new(percent / 100.0)
    }

    /// Returns the inner `f64`.
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }

    /// Returns the fraction as an `f64` percentage in `[0, 100]`.
    #[must_use]
    pub fn as_percent(self) -> f64 {
        self.0 * 100.0
    }
}

impl TryFrom<f64> for Fraction {
    type Error = FractionError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Fraction::new(value)
    }
}

impl From<Fraction> for f64 {
    fn from(f: Fraction) -> Self {
        f.0
    }
}

impl Mul<f64> for Fraction {
    type Output = f64;
    fn mul(self, rhs: f64) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<Fraction> for f64 {
    type Output = f64;
    fn mul(self, rhs: Fraction) -> Self::Output {
        self * rhs.0
    }
}

// Safe because `Fraction::new`/`TryFrom` forbid NaN and infinity.
impl Eq for Fraction {}

impl Ord for Fraction {
    /// Compares two `Fraction`s.
    ///
    /// Uses the underlying `f64`'s `partial_cmp` and unwraps the result.
    /// The unwrap is safe because `Fraction` guarantees values are finite
    /// and within `[0, 1]`, so `partial_cmp` always returns `Some(_)`.
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl PartialOrd for Fraction {
    /// Delegates to [`Ord::cmp`] to ensure a total, consistent ordering.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Errors that can occur when constructing a [`Fraction`].
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum FractionError {
    /// Input was not finite.
    #[error("value is not finite: {0}")]
    NotFinite(f64),

    /// Input was outside the allowed range.
    #[error("value {0} is outside the range [0, 1]")]
    OutOfRange(f64),
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn valid_values() {
        assert_eq!(Fraction::new(0.0).unwrap().get(), 0.0);
        assert_eq!(Fraction::new(1.0).unwrap().get(), 1.0);
        assert_eq!(Fraction::new(0.5).unwrap().get(), 0.5);
    }

    #[test]
    fn invalid_values() {
        assert!(matches!(
            Fraction::new(-0.01),
            Err(FractionError::OutOfRange(_))
        ));
        assert!(matches!(
            Fraction::new(1.01),
            Err(FractionError::OutOfRange(_))
        ));
        assert!(matches!(
            Fraction::new(f64::NAN),
            Err(FractionError::NotFinite(_))
        ));
        assert!(matches!(
            Fraction::new(f64::INFINITY),
            Err(FractionError::NotFinite(_))
        ));
        assert!(matches!(
            Fraction::new(f64::NEG_INFINITY),
            Err(FractionError::NotFinite(_))
        ));
    }

    #[test]
    fn percent_helpers() {
        let f = Fraction::from_percent(25.0).unwrap();
        assert_eq!(f.get(), 0.25);
        assert_eq!(f.as_percent(), 25.0);
    }

    #[test]
    fn mul_ergonomics() {
        let f = Fraction::new(0.25).unwrap();
        assert_eq!(f * 200.0, 50.0);
        assert_eq!(200.0 * f, 50.0);
    }

    #[test]
    fn try_from_and_into() {
        let f = Fraction::try_from(0.75).unwrap();
        let x: f64 = f.into();
        assert_eq!(x, 0.75);
    }
}
