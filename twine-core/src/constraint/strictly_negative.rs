use std::{cmp::Ordering, marker::PhantomData, ops::Add};

use num_traits::Zero;

use super::{Constrained, Constraint, ConstraintError};

/// Marker type enforcing that a value is strictly negative (less than zero).
///
/// Use this type with [`Constrained<T, StrictlyNegative>`] to encode strict
/// negativity at the type level.
///
/// You can construct a value constrained to be strictly negative using
/// either the generic [`Constrained::new`] method or the convenient
/// [`StrictlyNegative::new`] associated function.
///
/// # Examples
///
/// ```
/// use twine_core::constraint::{Constrained, StrictlyNegative};
///
/// // Generic constructor:
/// let x = Constrained::<_, StrictlyNegative>::new(-1).unwrap();
/// assert_eq!(x.into_inner(), -1);
///
/// // Associated constructor:
/// let y = StrictlyNegative::new(-2.5).unwrap();
/// assert_eq!(y.into_inner(), -2.5);
///
/// // Error cases:
/// assert!(StrictlyNegative::new(0).is_err());
/// assert!(StrictlyNegative::new(3).is_err());
/// assert!(StrictlyNegative::new(f64::NAN).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct StrictlyNegative;

impl StrictlyNegative {
    /// Constructs a [`Constrained<T, StrictlyNegative>`] if the value is strictly negative.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is zero, positive, or not a number (`NaN`).
    pub fn new<T: PartialOrd + Zero>(
        value: T,
    ) -> Result<Constrained<T, StrictlyNegative>, ConstraintError> {
        Constrained::<T, StrictlyNegative>::new(value)
    }
}

impl<T: PartialOrd + Zero> Constraint<T> for StrictlyNegative {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match value.partial_cmp(&T::zero()) {
            Some(Ordering::Less) => Ok(()),
            Some(Ordering::Equal) => Err(ConstraintError::Zero),
            Some(Ordering::Greater) => Err(ConstraintError::Negative),
            None => Err(ConstraintError::NotANumber),
        }
    }
}

/// Adds two `Constrained<T, StrictlyNegative>` values.
///
/// Assumes that summing two negative values yields a negative result.
/// This holds for most numeric types (`i32`, `f64`, `uom::Quantity`, etc.),
/// but may not for all possible `T`.
/// The invariant is checked in debug builds.
///
/// # Panics
///
/// Panics in debug builds if the sum is unexpectedly non-negative.
impl<T> Add for Constrained<T, StrictlyNegative>
where
    T: Add<Output = T> + PartialOrd + Zero,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let value = self.value + rhs.value;
        debug_assert!(
            value < T::zero(),
            "Addition produced a non-negative value, violating StrictlyNegative bound invariant"
        );
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers() {
        let x = Constrained::<i32, StrictlyNegative>::new(-1).unwrap();
        assert_eq!(x.into_inner(), -1);

        let y = StrictlyNegative::new(-42).unwrap();
        assert_eq!(y.as_ref(), &-42);

        assert!(StrictlyNegative::new(0).is_err());
        assert!(StrictlyNegative::new(2).is_err());
    }

    #[test]
    fn floats() {
        assert!(Constrained::<f64, StrictlyNegative>::new(-1.0).is_ok());
        assert!(StrictlyNegative::new(-0.1).is_ok());
        assert!(StrictlyNegative::new(0.0).is_err());
        assert!(StrictlyNegative::new(5.0).is_err());
        assert!(StrictlyNegative::new(f64::NAN).is_err());
    }
}
