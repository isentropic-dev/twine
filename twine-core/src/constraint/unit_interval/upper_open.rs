use std::{cmp::Ordering, marker::PhantomData};

use crate::constraint::{Constrained, Constraint, ConstraintError};

use super::UnitBounds;

/// Marker type enforcing that a value lies in the right-open unit interval: `0 ≤ x < 1`.
///
/// Requires `T: UnitBounds`.
/// We provide [`UnitBounds`] implementations for `f32`, `f64`, and `uom::si::f64::Ratio`.
///
/// You can construct a value constrained to `[0, 1)` using either the generic
/// [`Constrained::new`] method or the convenient [`UnitIntervalUpperOpen::new`]
/// associated function.
///
/// # Examples
///
/// Using with `f64`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitIntervalUpperOpen};
///
/// // Generic constructor:
/// let a = Constrained::<_, UnitIntervalUpperOpen>::new(0.25).unwrap();
/// assert_eq!(a.into_inner(), 0.25);
///
/// // Associated constructor:
/// let b = UnitIntervalUpperOpen::new(0.0).unwrap();
/// assert_eq!(b.as_ref(), &0.0);
///
/// // Endpoint:
/// let z = UnitIntervalUpperOpen::zero::<f64>();
/// assert_eq!(z.into_inner(), 0.0);
///
/// // Error cases:
/// assert!(UnitIntervalUpperOpen::new(1.0).is_err());
/// assert!(UnitIntervalUpperOpen::new(1.5).is_err());
/// assert!(UnitIntervalUpperOpen::new(f64::NAN).is_err());
/// ```
///
/// Using with `uom::si::f64::Ratio`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitIntervalUpperOpen};
/// use uom::si::{f64::Ratio, ratio::{ratio, percent}};
///
/// let r = Constrained::<Ratio, UnitIntervalUpperOpen>::new(Ratio::new::<ratio>(0.42)).unwrap();
/// assert_eq!(r.as_ref().get::<percent>(), 42.0);
///
/// let z = UnitIntervalUpperOpen::zero::<Ratio>();
/// assert_eq!(z.into_inner().get::<percent>(), 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitIntervalUpperOpen;

impl UnitIntervalUpperOpen {
    /// Constructs `Constrained<T, UnitIntervalUpperOpen>` if 0 ≤ value < 1.
    ///
    /// # Errors
    ///
    /// Fails if the value is outside the upper-open unit interval:
    ///
    /// - [`ConstraintError::BelowMinimum`] if less than zero.
    /// - [`ConstraintError::AboveMaximum`] if greater than or equal to one.
    /// - [`ConstraintError::NotANumber`] if comparison is undefined (e.g., NaN).
    pub fn new<T: UnitBounds>(
        value: T,
    ) -> Result<Constrained<T, UnitIntervalUpperOpen>, ConstraintError> {
        Constrained::<T, UnitIntervalUpperOpen>::new(value)
    }

    /// Returns the lower bound (zero) as a constrained value.
    #[must_use]
    pub fn zero<T: UnitBounds>() -> Constrained<T, UnitIntervalUpperOpen> {
        Constrained::<T, UnitIntervalUpperOpen> {
            value: T::zero(),
            _marker: PhantomData,
        }
    }
}

impl<T: UnitBounds> Constraint<T> for UnitIntervalUpperOpen {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match (value.partial_cmp(&T::zero()), value.partial_cmp(&T::one())) {
            (None, _) | (_, None) => Err(ConstraintError::NotANumber),
            (Some(Ordering::Less), _) => Err(ConstraintError::BelowMinimum),
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
        assert!(Constrained::<f64, UnitIntervalUpperOpen>::new(0.0).is_ok());
        assert!(Constrained::<f64, UnitIntervalUpperOpen>::new(0.9).is_ok());
        assert!(UnitIntervalUpperOpen::new(0.5).is_ok());

        let z = UnitIntervalUpperOpen::zero::<f64>();
        assert_eq!(z.into_inner(), 0.0);
    }

    #[test]
    fn floats_out_of_range() {
        assert!(matches!(
            UnitIntervalUpperOpen::new(-1.0),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalUpperOpen::new(1.0),
            Err(ConstraintError::AboveMaximum)
        ));
        assert!(matches!(
            UnitIntervalUpperOpen::new(2.0),
            Err(ConstraintError::AboveMaximum)
        ));
    }

    #[test]
    fn floats_nan_is_not_a_number() {
        assert!(matches!(
            UnitIntervalUpperOpen::new(f64::NAN),
            Err(ConstraintError::NotANumber)
        ));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn uom_ratio_valid() {
        assert!(Constrained::<Ratio, UnitIntervalUpperOpen>::new(Ratio::new::<ratio>(0.0)).is_ok());
        assert!(
            Constrained::<Ratio, UnitIntervalUpperOpen>::new(Ratio::new::<ratio>(0.99)).is_ok()
        );
        assert!(UnitIntervalUpperOpen::new(Ratio::new::<ratio>(0.5)).is_ok());

        let z = UnitIntervalUpperOpen::zero::<Ratio>();
        assert_eq!(z.into_inner().get::<ratio>(), 0.0);
    }

    #[test]
    fn uom_ratio_out_of_range() {
        assert!(matches!(
            UnitIntervalUpperOpen::new(Ratio::new::<ratio>(-0.1)),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitIntervalUpperOpen::new(Ratio::new::<ratio>(1.0)),
            Err(ConstraintError::AboveMaximum)
        ));
    }
}
