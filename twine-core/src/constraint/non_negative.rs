use std::{cmp::Ordering, marker::PhantomData, ops::Add};

use num_traits::Zero;

use super::{Constrained, Constraint, ConstraintError};

/// Marker type enforcing that a value is non-negative (zero or greater).
///
/// Use this type with [`Constrained<T, NonNegative>`] to encode non-negativity
/// at the type level.
///
/// You can construct a value constrained to be non-negative using either the
/// generic [`Constrained::new`] method or the convenient [`NonNegative::new`]
/// associated function.
///
/// # Examples
///
/// ```
/// use twine_core::constraint::{Constrained, NonNegative};
///
/// // Generic constructor:
/// let x = Constrained::<_, NonNegative>::new(5).unwrap();
/// assert_eq!(x.into_inner(), 5);
///
/// // Associated constructor:
/// let y = NonNegative::new(0.0).unwrap();
/// assert_eq!(y.into_inner(), 0.0);
///
/// // Error cases:
/// assert!(NonNegative::new(-7).is_err());
/// assert!(NonNegative::new(f64::NAN).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NonNegative;

impl NonNegative {
    /// Constructs a [`Constrained<T, NonNegative>`] if the value is non-negative.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is negative or not a number (`NaN`).
    pub fn new<T: PartialOrd + Zero>(
        value: T,
    ) -> Result<Constrained<T, NonNegative>, ConstraintError> {
        Constrained::<T, NonNegative>::new(value)
    }

    /// Returns the additive identity (zero) as a non-negative constrained value.
    ///
    /// This method is equivalent to [`Constrained::<T, NonNegative>::zero()`].
    #[must_use]
    pub fn zero<T: PartialOrd + Zero>() -> Constrained<T, NonNegative> {
        Constrained::<T, NonNegative>::zero()
    }
}

impl<T: PartialOrd + Zero> Constraint<T> for NonNegative {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match value.partial_cmp(&T::zero()) {
            Some(Ordering::Greater | Ordering::Equal) => Ok(()),
            Some(Ordering::Less) => Err(ConstraintError::Negative),
            None => Err(ConstraintError::NotANumber),
        }
    }
}

/// Adds two `Constrained<T, NonNegative>` values.
///
/// Assumes that summing two non-negative values yields a non-negative result.
/// This holds for most numeric types (`i32`, `f64`, `uom::Quantity`, etc.),
/// but may not for all possible `T`.
/// The invariant is checked in debug builds.
///
/// # Panics
///
/// Panics in debug builds if the sum is unexpectedly negative.
impl<T> Add for Constrained<T, NonNegative>
where
    T: Add<Output = T> + PartialOrd + Zero,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let value = self.value + rhs.value;
        debug_assert!(
            value >= T::zero(),
            "Addition produced a negative value, violating NonNegative bound invariant"
        );
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

impl<T> Zero for Constrained<T, NonNegative>
where
    T: PartialOrd + Zero,
{
    fn zero() -> Self {
        Self {
            value: T::zero(),
            _marker: PhantomData,
        }
    }

    fn is_zero(&self) -> bool {
        self.value == T::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{f64::MassRate, mass_rate::kilogram_per_second};

    #[test]
    fn integers() {
        let one = Constrained::<i32, NonNegative>::new(1).unwrap();
        assert_eq!(one.into_inner(), 1);

        let two = NonNegative::new(2).unwrap();
        assert_eq!(two.as_ref(), &2);

        let zero = NonNegative::zero();
        assert_eq!(zero.into_inner(), 0);

        let sum = one + two + zero;
        assert_eq!(sum.into_inner(), 3);

        assert!(NonNegative::new(-1).is_err());
    }

    #[test]
    fn floats() {
        assert!(Constrained::<f64, NonNegative>::new(2.0).is_ok());
        assert!(NonNegative::new(0.0).is_ok());
        assert!(NonNegative::new(-2.0).is_err());
        assert!(NonNegative::new(f64::NAN).is_err());
    }

    #[test]
    fn mass_rates() {
        let mass_rate = MassRate::new::<kilogram_per_second>(5.0);
        assert!(NonNegative::new(mass_rate).is_ok());

        let mass_rate = MassRate::new::<kilogram_per_second>(0.0);
        assert!(NonNegative::new(mass_rate).is_ok());

        let mass_rate = MassRate::new::<kilogram_per_second>(-2.0);
        assert!(NonNegative::new(mass_rate).is_err());
    }
}
