use std::ops::Add;

use num_traits::Zero;

/// A wrapper type representing values that are guaranteed to be non-negative.
///
/// `NonNegative<T>` is a lightweight newtype that wraps a value of type `T` and
/// enforces the invariant that the value is greater than or equal to zero.
/// This invariant is verified at construction time and preserved by all public
/// operations on the type.
///
/// # Type Constraints
///
/// `T` must implement both [`PartialOrd`] and [`Zero`].
/// Common examples include primitive numeric types (`i32`, `f64`, etc.) and
/// unit-safe types like `Quantity` from [`uom`].
///
/// # Examples
///
/// ```
/// use twine_core::NonNegative;
///
/// let x = NonNegative::new(3).unwrap();
/// assert_eq!(x.into_inner(), 3);
///
/// assert!(NonNegative::new(-5).is_none());
/// ```
///
/// [`PartialOrd`]: https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html
/// [`Zero`]: https://docs.rs/num-traits/latest/num_traits/identities/trait.Zero.html
/// [`uom`]: https://docs.rs/uom/
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonNegative<T>(T);

impl<T> NonNegative<T>
where
    T: PartialOrd + Zero,
{
    /// Constructs a new `NonNegative<T>` if the input value is non-negative.
    ///
    /// Returns `Some(Self)` if `value >= 0`, or `None` otherwise.
    pub fn new(value: T) -> Option<Self> {
        if value >= T::zero() {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Returns the additive identity (zero) wrapped as a `NonNegative`.
    ///
    /// Equivalent to `NonNegative::new(T::zero()).unwrap()`, but more ergonomic
    /// and avoids the overhead of a runtime check.
    #[must_use]
    pub fn zero() -> Self {
        Self(T::zero())
    }
}

impl<T> AsRef<T> for NonNegative<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// Adds two `NonNegative<T>` values.
///
/// Assumes that summing two non-negative values yields a non-negative result.
/// This holds for most numeric types (`u32`, `f64`, `uom::Quantity`, etc.)
/// but may not for all possible `T`.
/// The invariant is checked in debug builds.
///
/// # Panics
///
/// Panics in debug builds if the sum is unexpectedly negative.
impl<T> Add for NonNegative<T>
where
    T: Add<Output = T> + PartialOrd + Zero,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let result = self.0 + rhs.0;
        debug_assert!(
            result >= T::zero(),
            "Addition produced a negative value, violating NonNegative invariant"
        );
        Self(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::{MassRate, Power},
        mass_rate::kilogram_per_second,
        power::watt,
    };

    #[test]
    fn non_negative_integers() {
        let one = NonNegative::new(1).unwrap();
        assert_eq!(one.into_inner(), 1);

        let zero = NonNegative::new(0).unwrap();
        assert_eq!(zero.as_ref(), &0);

        let two = one + one + zero;
        assert_eq!(two.into_inner(), 2);

        assert!(NonNegative::new(-1).is_none(), "A negative value is not ok");
    }

    #[test]
    fn non_negative_floats() {
        assert!(NonNegative::new(2.0).is_some(), "Positive value is ok");
        assert!(NonNegative::new(0.0).is_some(), "Zero value is ok");
        assert!(NonNegative::new(-2.0).is_none(), "Negative value is not ok",);
        assert!(NonNegative::new(f64::NAN).is_none(), "NaN is not ok");
    }

    #[test]
    fn non_negative_mass_rate() {
        let mass_rate = MassRate::new::<kilogram_per_second>(5.0);
        assert!(
            NonNegative::new(mass_rate).is_some(),
            "A positive mass rate is ok",
        );

        let mass_rate = MassRate::new::<kilogram_per_second>(0.0);
        assert!(
            NonNegative::new(mass_rate).is_some(),
            "A zero mass rate is ok",
        );

        let mass_rate = MassRate::new::<kilogram_per_second>(-2.0);
        assert!(
            NonNegative::new(mass_rate).is_none(),
            "A negative mass rate is not ok",
        );
    }

    #[test]
    fn non_negative_power() {
        let positive_power = Power::new::<watt>(5.0);
        assert!(
            NonNegative::new(positive_power).is_some(),
            "A positive power value is ok",
        );

        let zero_power = Power::new::<watt>(0.0);
        assert!(
            NonNegative::new(zero_power).is_some(),
            "A zero power value is ok"
        );

        let negative_power = Power::new::<watt>(-2.0);
        assert!(
            NonNegative::new(negative_power).is_none(),
            "A negative power value is not ok",
        );
    }
}
