use std::cmp::Ordering;

use crate::constraint::{Constrained, Constraint, ConstraintError};

use super::UnitBounds;

/// Marker type enforcing that a value lies in the open unit interval: `0 < x < 1`.
///
/// Requires `T: UnitBounds`.
/// We provide [`UnitBounds`] implementations for `f32`, `f64`, and `uom::si::f64::Ratio`.
///
/// You can construct a value constrained to `(0, 1)` using either the generic
/// [`Constrained::new`] method or the convenient [`UnitIntervalOpen::new`]
/// associated function.
///
/// # Examples
///
/// Using with `f64`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitIntervalOpen};
///
/// // Generic constructor:
/// let a = Constrained::<_, UnitIntervalOpen>::new(0.25).unwrap();
/// assert_eq!(a.into_inner(), 0.25);
///
/// // Associated constructor:
/// let b = UnitIntervalOpen::new(0.9).unwrap();
/// assert_eq!(b.as_ref(), &0.9);
///
/// // Error cases:
/// assert!(UnitIntervalOpen::new(0.0).is_err());
/// assert!(UnitIntervalOpen::new(1.0).is_err());
/// assert!(UnitIntervalOpen::new(f64::NAN).is_err());
/// ```
///
/// Using with `uom::si::f64::Ratio`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitIntervalOpen};
/// use uom::si::{f64::Ratio, ratio::{ratio, percent}};
///
/// let r = Constrained::<Ratio, UnitIntervalOpen>::new(Ratio::new::<ratio>(0.42)).unwrap();
/// assert_eq!(r.as_ref().get::<percent>(), 42.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitIntervalOpen;

impl UnitIntervalOpen {
    /// Constructs `Constrained<T, UnitIntervalOpen>` if 0 < value < 1.
    ///
    /// # Errors
    ///
    /// Fails if the value is outside the open unit interval:
    ///
    /// - [`ConstraintError::BelowMinimum`] if less than or equal to zero.
    /// - [`ConstraintError::AboveMaximum`] if greater than or equal to one.
    /// - [`ConstraintError::NotANumber`] if comparison is undefined (e.g., NaN).
    pub fn new<T: UnitBounds>(
        value: T,
    ) -> Result<Constrained<T, UnitIntervalOpen>, ConstraintError> {
        Constrained::<T, UnitIntervalOpen>::new(value)
    }
}

impl<T: UnitBounds> Constraint<T> for UnitIntervalOpen {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match (value.partial_cmp(&T::zero()), value.partial_cmp(&T::one())) {
            (None, _) | (_, None) => Err(ConstraintError::NotANumber),
            (Some(Ordering::Less | Ordering::Equal), _) => Err(ConstraintError::BelowMinimum),
            (_, Some(Ordering::Greater | Ordering::Equal)) => Err(ConstraintError::AboveMaximum),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{f64::Ratio, ratio::ratio};

    #[test]
    #[allow(clippy::float_cmp)]
    fn floats_valid() {
        assert!(Constrained::<f64, UnitIntervalOpen>::new(0.1).is_ok());
        assert!(Constrained::<f64, UnitIntervalOpen>::new(0.9).is_ok());
        assert!(UnitIntervalOpen::new(0.5).is_ok());
    }

    #[test]
    fn floats_out_of_range() {
        assert!(matches!(
            UnitIntervalOpen::new(0.0),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalOpen::new(-1.0),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalOpen::new(1.0),
            Err(ConstraintError::AboveMaximum)
        ));
        assert!(matches!(
            UnitIntervalOpen::new(2.0),
            Err(ConstraintError::AboveMaximum)
        ));
    }

    #[test]
    fn floats_nan_is_not_a_number() {
        assert!(matches!(
            UnitIntervalOpen::new(f64::NAN),
            Err(ConstraintError::NotANumber)
        ));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn uom_ratio_valid() {
        assert!(Constrained::<Ratio, UnitIntervalOpen>::new(Ratio::new::<ratio>(0.01)).is_ok());
        assert!(Constrained::<Ratio, UnitIntervalOpen>::new(Ratio::new::<ratio>(0.99)).is_ok());
        assert!(UnitIntervalOpen::new(Ratio::new::<ratio>(0.5)).is_ok());
    }

    #[test]
    fn uom_ratio_out_of_range() {
        assert!(matches!(
            UnitIntervalOpen::new(Ratio::new::<ratio>(0.0)),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalOpen::new(Ratio::new::<ratio>(1.0)),
            Err(ConstraintError::AboveMaximum)
        ));
    }
}
