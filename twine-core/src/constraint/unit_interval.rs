use std::{cmp::Ordering, marker::PhantomData};

use uom::si::{f64::Ratio, ratio::ratio};

use super::{Constrained, Constraint, ConstraintError};

/// Marker type enforcing that a value lies in the closed unit interval: `0 ≤ x ≤ 1`.
///
/// Requires `T: UnitBounds`.
/// We provide [`UnitBounds`] implementations for `f32`, `f64`, and `uom::si::f64::Ratio`.
///
/// You can construct a value constrained to `[0, 1]` using either the generic
/// [`Constrained::new`] method or the convenient [`UnitInterval::new`]
/// associated function.
/// Convenience constructors [`UnitInterval::zero`] and [`UnitInterval::one`]
/// are also provided for the endpoints.
///
/// # Examples
///
/// Using with `f64`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitInterval};
///
/// // Generic constructor:
/// let a = Constrained::<_, UnitInterval>::new(0.25).unwrap();
/// assert_eq!(a.into_inner(), 0.25);
///
/// // Associated constructor:
/// let b = UnitInterval::new(1.0).unwrap();
/// assert_eq!(b.as_ref(), &1.0);
///
/// // Endpoints:
/// let z = UnitInterval::zero::<f64>();
/// let o = UnitInterval::one::<f64>();
/// assert_eq!((z.into_inner(), o.into_inner()), (0.0, 1.0));
///
/// // Error cases:
/// assert!(UnitInterval::new(-0.0001).is_err());
/// assert!(UnitInterval::new(1.0001).is_err());
/// assert!(UnitInterval::new(f64::NAN).is_err());
/// ```
///
/// Using with `uom::si::f64::Ratio`:
///
/// ```
/// use twine_core::constraint::{Constrained, UnitInterval};
/// use uom::si::{f64::Ratio, ratio::{ratio, percent}};
///
/// let r = Constrained::<Ratio, UnitInterval>::new(Ratio::new::<ratio>(0.42)).unwrap();
/// assert_eq!(r.as_ref().get::<percent>(), 42.0);
///
/// let z = UnitInterval::zero::<Ratio>();
/// let o = UnitInterval::one::<Ratio>();
/// assert_eq!(z.into_inner().get::<ratio>(), 0.0);
/// assert_eq!(o.into_inner().get::<ratio>(), 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitInterval;

impl UnitInterval {
    /// Constructs `Constrained<T, UnitInterval>` if 0 ≤ value ≤ 1.
    ///
    /// # Errors
    ///
    /// Fails if the value is outside the closed unit interval:
    ///
    /// - [`ConstraintError::BelowMinimum`] if less than zero.
    /// - [`ConstraintError::AboveMaximum`] if greater than one.
    /// - [`ConstraintError::NotANumber`] if comparison is undefined (e.g., NaN).
    pub fn new<T: UnitBounds>(value: T) -> Result<Constrained<T, UnitInterval>, ConstraintError> {
        Constrained::<T, UnitInterval>::new(value)
    }

    /// Returns the lower bound (zero) as a constrained value.
    #[must_use]
    pub fn zero<T: UnitBounds>() -> Constrained<T, UnitInterval> {
        Constrained::<T, UnitInterval> {
            value: T::zero(),
            _marker: PhantomData,
        }
    }

    /// Returns the upper bound (one) as a constrained value.
    #[must_use]
    pub fn one<T: UnitBounds>() -> Constrained<T, UnitInterval> {
        Constrained::<T, UnitInterval> {
            value: T::one(),
            _marker: PhantomData,
        }
    }
}

impl<T: UnitBounds> Constraint<T> for UnitInterval {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match (value.partial_cmp(&T::zero()), value.partial_cmp(&T::one())) {
            (None, _) | (_, None) => Err(ConstraintError::NotANumber),
            (Some(Ordering::Less), _) => Err(ConstraintError::BelowMinimum),
            (_, Some(Ordering::Greater)) => Err(ConstraintError::AboveMaximum),
            _ => Ok(()),
        }
    }
}

/// Supplies 0 and 1 for types used in the closed unit interval [0, 1].
///
/// Implement this trait for your type `T` if you want to use it with
/// `Constrained<T, UnitInterval>`.
/// Implementations should ensure that `zero() <= one()` under the type's
/// `PartialOrd` so the closed interval is well-formed.
pub trait UnitBounds: PartialOrd {
    fn zero() -> Self;
    fn one() -> Self;
}

impl UnitBounds for f32 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl UnitBounds for f64 {
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
}

impl UnitBounds for Ratio {
    fn zero() -> Self {
        Ratio::new::<ratio>(0.0)
    }
    fn one() -> Self {
        Ratio::new::<ratio>(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::ratio::percent;

    #[test]
    #[allow(clippy::float_cmp)]
    fn floats_valid() {
        assert!(Constrained::<f64, UnitInterval>::new(0.0).is_ok());
        assert!(Constrained::<f64, UnitInterval>::new(1.0).is_ok());
        assert!(UnitInterval::new(0.5).is_ok());

        let z = UnitInterval::zero::<f64>();
        let o = UnitInterval::one::<f64>();
        assert_eq!(z.into_inner(), 0.0);
        assert_eq!(o.into_inner(), 1.0);
    }

    #[test]
    fn floats_out_of_range() {
        assert!(matches!(
            UnitInterval::new(-1.0),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitInterval::new(2.0),
            Err(ConstraintError::AboveMaximum)
        ));
        assert!(matches!(
            UnitInterval::new(-1e-15),
            Err(ConstraintError::BelowMinimum),
        ));
        assert!(matches!(
            UnitInterval::new(1.0 + 1e-15),
            Err(ConstraintError::AboveMaximum)
        ));
        assert!(matches!(
            UnitInterval::new(f64::INFINITY),
            Err(ConstraintError::AboveMaximum)
        ));
        assert!(matches!(
            UnitInterval::new(f64::NEG_INFINITY),
            Err(ConstraintError::BelowMinimum)
        ));
    }

    #[test]
    fn floats_nan_is_not_a_number() {
        assert!(matches!(
            UnitInterval::new(f64::NAN),
            Err(ConstraintError::NotANumber)
        ));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn uom_ratio_valid() {
        assert!(Constrained::<Ratio, UnitInterval>::new(Ratio::new::<ratio>(0.0)).is_ok());
        assert!(Constrained::<Ratio, UnitInterval>::new(Ratio::new::<ratio>(1.0)).is_ok());
        assert!(UnitInterval::new(Ratio::new::<ratio>(0.5)).is_ok());

        let z = UnitInterval::zero::<Ratio>();
        let o = UnitInterval::one::<Ratio>();

        assert_eq!(z.into_inner().get::<ratio>(), 0.0);
        assert_eq!(z.into_inner().get::<percent>(), 0.0);

        assert_eq!(o.into_inner().get::<ratio>(), 1.0);
        assert_eq!(o.into_inner().get::<percent>(), 100.0);
    }

    #[test]
    fn uom_ratio_out_of_range() {
        assert!(matches!(
            UnitInterval::new(Ratio::new::<ratio>(-0.1)),
            Err(ConstraintError::BelowMinimum)
        ));
        assert!(matches!(
            UnitInterval::new(Ratio::new::<ratio>(1.1)),
            Err(ConstraintError::AboveMaximum)
        ));
    }
}
