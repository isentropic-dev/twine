use std::{cmp::Ordering, marker::PhantomData, ops::Add};

use num_traits::Zero;

use super::{Constrained, Constraint, ConstraintError};

/// Marker type enforcing that a value is non-positive (zero or less).
///
/// Use this type with [`Constrained<T, NonPositive>`] to encode non-positivity
/// at the type level.
///
/// You can construct a value constrained to be non-positive using either the
/// generic [`Constrained::new`] method or the convenient [`NonPositive::new`]
/// associated function.
///
/// # Examples
///
/// ```
/// use twine_core::constraint::{Constrained, NonPositive};
///
/// // Generic constructor:
/// let x = Constrained::<_, NonPositive>::new(0).unwrap();
/// assert_eq!(x.into_inner(), 0);
///
/// // Associated constructor:
/// let y = NonPositive::new(-5).unwrap();
/// assert_eq!(y.into_inner(), -5);
///
/// // Error cases:
/// assert!(NonPositive::new(3).is_err());
/// assert!(NonPositive::new(f64::NAN).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonPositive;

impl NonPositive {
    /// Constructs a [`Constrained<T, NonPositive>`] if the value is non-positive.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is positive or not a number (`NaN`).
    pub fn new<T: PartialOrd + Zero>(
        value: T,
    ) -> Result<Constrained<T, NonPositive>, ConstraintError> {
        Constrained::<T, NonPositive>::new(value)
    }

    /// Returns the additive identity (zero) as a non-positive constrained value.
    ///
    /// This method is equivalent to [`Constrained::<T, NonPositive>::zero()`].
    #[must_use]
    pub fn zero<T: PartialOrd + Zero>() -> Constrained<T, NonPositive> {
        Constrained::<T, NonPositive>::zero()
    }
}

impl<T: PartialOrd + Zero> Constraint<T> for NonPositive {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match value.partial_cmp(&T::zero()) {
            Some(Ordering::Less | Ordering::Equal) => Ok(()),
            Some(Ordering::Greater) => Err(ConstraintError::Positive),
            None => Err(ConstraintError::NotANumber),
        }
    }
}

/// Adds two `Constrained<T, NonPositive>` values.
///
/// Assumes that summing two non-positive values yields a non-positive result.
/// This holds for most numeric types (`i32`, `f64`, `uom::Quantity`, etc.),
/// but may not for all possible `T`.
/// The invariant is checked in debug builds.
///
/// # Panics
///
/// Panics in debug builds if the sum is unexpectedly positive.
impl<T> Add for Constrained<T, NonPositive>
where
    T: Add<Output = T> + PartialOrd + Zero,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let value = self.value + rhs.value;
        debug_assert!(
            value <= T::zero(),
            "Addition produced a positive value, violating NonPositive bound invariant"
        );
        Self {
            value,
            _marker: PhantomData,
        }
    }
}

impl<T> Zero for Constrained<T, NonPositive>
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
    use uom::si::{f64::Power, power::watt};

    #[test]
    fn integers() {
        let neg_one = Constrained::<i32, NonPositive>::new(-1).unwrap();
        assert_eq!(neg_one.into_inner(), -1);

        let neg_two = NonPositive::new(-2).unwrap();
        assert_eq!(neg_two.as_ref(), &-2);

        let zero = NonPositive::zero();
        assert_eq!(zero.into_inner(), 0);

        let sum = neg_one + neg_two + zero;
        assert_eq!(sum.into_inner(), -3);

        assert!(NonPositive::new(2).is_err());
    }

    #[test]
    fn floats() {
        assert!(Constrained::<f64, NonPositive>::new(-2.0).is_ok());
        assert!(NonPositive::new(0.0).is_ok());
        assert!(NonPositive::new(2.0).is_err());
        assert!(NonPositive::new(f64::NAN).is_err());
    }

    #[test]
    fn powers() {
        let neg_mass_rate = Power::new::<watt>(-5.0);
        assert!(NonPositive::new(neg_mass_rate).is_ok());

        let zero_mass_rate = Power::new::<watt>(0.0);
        assert!(NonPositive::new(zero_mass_rate).is_ok());

        let pos_mass_rate = Power::new::<watt>(2.0);
        assert!(NonPositive::new(pos_mass_rate).is_err());
    }
}
