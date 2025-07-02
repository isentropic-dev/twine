use std::cmp::Ordering;

use num_traits::Zero;

use super::{Constrained, Constraint, ConstraintError};

/// Marker type enforcing that a value is non-zero (not equal to zero).
///
/// Use this type with [`Constrained<T, NonZero>`] to encode a non-zero
/// constraint at the type level.
///
/// You can construct a value constrained to be non-zero using either the
/// generic [`Constrained::new`] method or the convenient [`NonZero::new`]
/// associated function.
///
/// # Examples
///
/// ```
/// use twine_core::constraint::{Constrained, NonZero};
///
/// // Generic constructor:
/// let x = Constrained::<_, NonZero>::new(1).unwrap();
/// assert_eq!(x.into_inner(), 1);
///
/// // Associated constructor:
/// let y = NonZero::new(-5.0).unwrap();
/// assert_eq!(y.into_inner(), -5.0);
///
/// // Error cases:
/// assert!(NonZero::new(0).is_err());
/// assert!(NonZero::new(0.0).is_err());
/// assert!(NonZero::new(f64::NAN).is_err());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NonZero;

impl NonZero {
    /// Constructs a [`Constrained<T, NonZero>`] if the value is not zero.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is zero or not a number (`NaN`).
    pub fn new<T: PartialOrd + Zero>(value: T) -> Result<Constrained<T, NonZero>, ConstraintError> {
        Constrained::<T, NonZero>::new(value)
    }
}

impl<T: PartialOrd + Zero> Constraint<T> for NonZero {
    fn check(value: &T) -> Result<(), ConstraintError> {
        match value.partial_cmp(&T::zero()) {
            Some(Ordering::Greater | Ordering::Less) => Ok(()),
            Some(Ordering::Equal) => Err(ConstraintError::Zero),
            None => Err(ConstraintError::NotANumber),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers() {
        let one = Constrained::<_, NonZero>::new(1).unwrap();
        assert_eq!(one.into_inner(), 1);

        let neg_one = NonZero::new(-1).unwrap();
        assert_eq!(neg_one.as_ref(), &-1);

        assert!(NonZero::new(0).is_err());
    }

    #[test]
    fn floats() {
        assert!(Constrained::<f64, NonZero>::new(2.0).is_ok());
        assert!(NonZero::new(-3.5).is_ok());
        assert!(NonZero::new(0.0).is_err());
        assert!(NonZero::new(f64::NAN).is_err());
    }
}
