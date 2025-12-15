use std::{cmp::Ordering, marker::PhantomData};

use crate::constraint::{Constrained, Constraint, ConstraintError};

use super::UnitBounds;

/// Marker type enforcing that a value lies in the left-open unit interval: `0 < x ≤ 1`.
///
/// Requires `T: UnitBounds`.
/// We provide [`UnitBounds`] implementations for `f32`, `f64`, and `uom::si::f64::Ratio`.
///
/// You can construct a value constrained to `(0, 1]` using either the generic
/// [`Constrained::new`] method or the convenient [`UnitIntervalLowerOpen::new`]
/// associated function.
///
/// # Examples
///
/// Using with `f64`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitIntervalLowerOpen};
///
/// // Generic constructor:
/// let a = Constrained::<_, UnitIntervalLowerOpen>::new(0.25).unwrap();
/// assert_eq!(a.into_inner(), 0.25);
///
/// // Associated constructor:
/// let b = UnitIntervalLowerOpen::new(1.0).unwrap();
/// assert_eq!(b.as_ref(), &1.0);
///
/// // Endpoint:
/// let o = UnitIntervalLowerOpen::one::<f64>();
/// assert_eq!(o.into_inner(), 1.0);
///
/// // Error cases:
/// assert!(UnitIntervalLowerOpen::new(0.0).is_err());
/// assert!(UnitIntervalLowerOpen::new(-0.5).is_err());
/// assert!(UnitIntervalLowerOpen::new(f64::NAN).is_err());
/// ```
///
/// Using with `uom::si::f64::Ratio`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitIntervalLowerOpen};
/// use uom::si::{f64::Ratio, ratio::{ratio, percent}};
///
/// let r = Constrained::<Ratio, UnitIntervalLowerOpen>::new(Ratio::new::<ratio>(0.42)).unwrap();
/// assert_eq!(r.as_ref().get::<percent>(), 42.0);
///
/// let o = UnitIntervalLowerOpen::one::<Ratio>();
/// assert_eq!(o.into_inner().get::<percent>(), 100.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitIntervalLowerOpen;

impl UnitIntervalLowerOpen {
    /// Constructs `Constrained<T, UnitIntervalLowerOpen>` if 0 < value ≤ 1.
    ///
    /// # Errors
    ///
    /// Fails if the value is outside the lower-open unit interval:
    ///
    /// - [`ConstraintError::BelowMinimum`] if less than or equal to zero.
    /// - [`ConstraintError::AboveMaximum`] if greater than one.
    /// - [`ConstraintError::NotANumber`] if comparison is undefined (e.g., NaN).
    pub fn new<T: UnitBounds>(
        value: T,
    ) -> Result<Constrained<T, UnitIntervalLowerOpen>, ConstraintError> {
        Constrained::<T, UnitIntervalLowerOpen>::new(value)
    }

    /// Returns the upper bound (one) as a constrained value.
    #[must_use]
    pub fn one<T: UnitBounds>() -> Constrained<T, UnitIntervalLowerOpen> {
        Constrained::<T, UnitIntervalLowerOpen> {
            value: T::one(),
            _marker: PhantomData,
        }
    }
}

impl<T: UnitBounds> Constraint<T> for UnitIntervalLowerOpen {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match (value.partial_cmp(&T::zero()), value.partial_cmp(&T::one())) {
            (None, _) | (_, None) => Err(ConstraintError::NotANumber),
            (Some(Ordering::Less | Ordering::Equal), _) => Err(ConstraintError::BelowMinimum),
            (_, Some(Ordering::Greater)) => Err(ConstraintError::AboveMaximum),
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
        assert!(Constrained::<f64, UnitIntervalLowerOpen>::new(0.1).is_ok());
        assert!(Constrained::<f64, UnitIntervalLowerOpen>::new(1.0).is_ok());
        assert!(UnitIntervalLowerOpen::new(0.75).is_ok());

        let o = UnitIntervalLowerOpen::one::<f64>();
        assert_eq!(o.into_inner(), 1.0);
    }

    #[test]
    fn floats_out_of_range() {
        assert!(matches!(
            UnitIntervalLowerOpen::new(0.0),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalLowerOpen::new(-1.0),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalLowerOpen::new(1.000_000_1),
            Err(ConstraintError::AboveMaximum)
        ));
    }

    #[test]
    fn floats_nan_is_not_a_number() {
        assert!(matches!(
            UnitIntervalLowerOpen::new(f64::NAN),
            Err(ConstraintError::NotANumber)
        ));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn uom_ratio_valid() {
        assert!(
            Constrained::<Ratio, UnitIntervalLowerOpen>::new(Ratio::new::<ratio>(0.01)).is_ok()
        );
        assert!(Constrained::<Ratio, UnitIntervalLowerOpen>::new(Ratio::new::<ratio>(1.0)).is_ok());
        assert!(UnitIntervalLowerOpen::new(Ratio::new::<ratio>(0.5)).is_ok());

        let o = UnitIntervalLowerOpen::one::<Ratio>();
        assert_eq!(o.into_inner().get::<ratio>(), 1.0);
    }

    #[test]
    fn uom_ratio_out_of_range() {
        assert!(matches!(
            UnitIntervalLowerOpen::new(Ratio::new::<ratio>(0.0)),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalLowerOpen::new(Ratio::new::<ratio>(1.1)),
            Err(ConstraintError::AboveMaximum)
        ));
    }
}
