use std::{cmp::Ordering, marker::PhantomData, ops::Add};

use num_traits::Zero;

use super::{Constrained, Constraint, ConstraintError};

/// Marker type enforcing that a value is strictly positive (greater than zero).
///
/// Use this type with [`Constrained<T, StrictlyPositive>`] to encode strict
/// positivity at the type level.
///
/// You can construct a value constrained to be strictly positive using
/// either the generic [`Constrained::new`] method or the convenient
/// [`StrictlyPositive::new`] associated function.
///
/// # Examples
///
/// ```
/// use twine_core::constraint::{Constrained, StrictlyPositive};
///
/// // Generic constructor:
/// let x = Constrained::<_, StrictlyPositive>::new(1).unwrap();
/// assert_eq!(x.into_inner(), 1);
///
/// // Associated constructor:
/// let y = StrictlyPositive::new(3.14).unwrap();
/// assert_eq!(y.into_inner(), 3.14);
///
/// // Error cases:
/// assert!(StrictlyPositive::new(0).is_err());
/// assert!(StrictlyPositive::new(-1).is_err());
/// assert!(StrictlyPositive::new(f64::NAN).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StrictlyPositive;

impl StrictlyPositive {
    /// Constructs a [`Constrained<T, StrictlyPositive>`] if the value is strictly positive.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is zero, negative, or not a number (`NaN`).
    pub fn new<T: PartialOrd + Zero>(
        value: T,
    ) -> Result<Constrained<T, StrictlyPositive>, ConstraintError> {
        Constrained::<T, StrictlyPositive>::new(value)
    }
}

impl<T: PartialOrd + Zero> Constraint<T> for StrictlyPositive {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match value.partial_cmp(&T::zero()) {
            Some(Ordering::Greater) => Ok(()),
            Some(Ordering::Equal) => Err(ConstraintError::Zero),
            Some(Ordering::Less) => Err(ConstraintError::Negative),
            None => Err(ConstraintError::NotANumber),
        }
    }
}

/// Adds two `Constrained<T, StrictlyPositive>` values.
///
/// Assumes that summing two positive values yields a positive result.
/// This holds for most numeric types (`i32`, `f64`, `uom::Quantity`, etc.),
/// but may not for all possible `T`.
/// The invariant is checked in debug builds.
///
/// # Panics
///
/// Panics in debug builds if the sum is unexpectedly non-positive.
impl<T> Add for Constrained<T, StrictlyPositive>
where
    T: Add<Output = T> + PartialOrd + Zero,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let value = self.value + rhs.value;
        debug_assert!(
            value > T::zero(),
            "Addition produced a non-positive value, violating StrictlyPositive bound invariant"
        );
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{f64::MassRate, mass_rate::kilogram_per_second};

    use super::*;

    #[test]
    fn integers() {
        let x = Constrained::<i32, StrictlyPositive>::new(1).unwrap();
        assert_eq!(x.into_inner(), 1);

        let y = StrictlyPositive::new(42).unwrap();
        assert_eq!(y.as_ref(), &42);

        assert!(StrictlyPositive::new(0).is_err());
        assert!(StrictlyPositive::new(-2).is_err());
    }

    #[test]
    fn floats() {
        assert!(Constrained::<f64, StrictlyPositive>::new(1.0).is_ok());
        assert!(StrictlyPositive::new(0.1).is_ok());
        assert!(StrictlyPositive::new(0.0).is_err());
        assert!(StrictlyPositive::new(-5.0).is_err());
        assert!(StrictlyPositive::new(f64::NAN).is_err());
    }

    #[test]
    fn mass_rates() {
        let mass_rate = MassRate::new::<kilogram_per_second>(5.0);
        assert!(StrictlyPositive::new(mass_rate).is_ok());

        let mass_rate = MassRate::new::<kilogram_per_second>(0.0);
        assert!(StrictlyPositive::new(mass_rate).is_err());

        let mass_rate = MassRate::new::<kilogram_per_second>(-2.0);
        assert!(StrictlyPositive::new(mass_rate).is_err());
    }
}
