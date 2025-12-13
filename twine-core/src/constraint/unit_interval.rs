mod closed;
mod left_open;
mod open;
mod right_open;

use uom::si::{f64::Ratio, ratio::ratio};

pub use closed::UnitInterval;
pub use left_open::UnitIntervalLeftOpen;
pub use open::UnitIntervalOpen;
pub use right_open::UnitIntervalRightOpen;

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
