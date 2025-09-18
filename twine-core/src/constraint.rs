//! Type-level numeric constraints with zero runtime cost.
//!
//! This module lets you express numeric constraints like "non-negative",
//! "non-zero", or "strictly positive" at the type level, with zero runtime
//! overhead after construction.
//!
//! With these types, your APIs and components can trust that values always
//! satisfy the required numeric constraints.
//! This guarantee leads to designs that are both safer and more self-documenting.
//!
//! # Provided Constraints
//!
//! The following marker types represent the most common numeric invariants:
//!
//! - [`NonNegative`]: Zero or greater
//! - [`NonPositive`]: Zero or less
//! - [`NonZero`]: Not equal to zero
//! - [`StrictlyNegative`]: Less than zero
//! - [`StrictlyPositive`]: Greater than zero
//!
//! Each marker can be used with the generic [`Constrained<T, C>`] wrapper,
//! where `C` is the marker type implementing [`Constraint<T>`].
//! Each also provides an associated [`new()`] constructor for convenience.
//!
//! See the documentation and tests for each constraint for more usage patterns.
//!
//! # Extending
//!
//! You can define custom numeric invariants by implementing [`Constraint<T>`]
//! for your own zero-sized marker types.

mod non_negative;
mod non_positive;
mod non_zero;
mod strictly_negative;
mod strictly_positive;
mod unit_interval;

use std::{iter::Sum, marker::PhantomData, ops::Add};

use num_traits::Zero;
use thiserror::Error;

pub use non_negative::NonNegative;
pub use non_positive::NonPositive;
pub use non_zero::NonZero;
pub use strictly_negative::StrictlyNegative;
pub use strictly_positive::StrictlyPositive;
pub use unit_interval::{UnitBounds, UnitInterval};

/// A trait for enforcing numeric invariants at construction time.
///
/// Implement this trait for any marker type representing a numeric constraint,
/// such as [`NonNegative`] or [`StrictlyPositive`].
pub trait Constraint<T> {
    /// Checks that the given value satisfies this constraint.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if the value does not satisfy the constraint.
    fn check(value: &T) -> Result<(), ConstraintError>;
}

/// An error returned when a [`Constraint`] is violated.
///
/// This enum is marked `#[non_exhaustive]` and may include additional variants
/// in future releases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ConstraintError {
    #[error("value must not be negative")]
    Negative,
    #[error("value must not be positive")]
    Positive,
    #[error("value must not be zero")]
    Zero,
    #[error("value is not a number")]
    NotANumber,
    #[error("value is below the minimum allowed")]
    BelowMinimum,
    #[error("value is above the maximum allowed")]
    AboveMaximum,
}

/// A result type alias to use with [`Constraint`].
pub type ConstraintResult<T, E = ConstraintError> = Result<T, E>;

/// A wrapper enforcing a numeric constraint at construction time.
///
/// Combine this with one of the provided marker types (such as [`NonNegative`])
/// or your own [`Constraint<T>`] implementation.
///
/// See the [module documentation](crate::constraint) for details and usage patterns.
///
/// # Example
///
/// ```
/// use twine_core::constraint::{Constrained, StrictlyPositive};
///
/// let n = Constrained::<_, StrictlyPositive>::new(42).unwrap();
/// assert_eq!(n.into_inner(), 42);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constrained<T, C: Constraint<T>> {
    value: T,
    _marker: PhantomData<C>,
}

impl<T, C: Constraint<T>> Constrained<T, C> {
    /// Constructs a new constrained value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value does not satisfy the constraint.
    pub fn new(value: T) -> Result<Self, ConstraintError> {
        C::check(&value)?;
        Ok(Self {
            value,
            _marker: PhantomData,
        })
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

/// Returns a reference to the inner unconstrained value.
impl<T, C: Constraint<T>> AsRef<T> for Constrained<T, C> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

/// Sums constrained values for which addition is valid.
///
/// Applies to all constraints that are preserved under addition.
impl<T, C> Sum for Constrained<T, C>
where
    C: Constraint<T>,
    Constrained<T, C>: Add<Output = Self> + Zero,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |a, b| a + b)
    }
}
