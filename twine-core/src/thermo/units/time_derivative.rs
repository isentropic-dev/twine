use std::ops::Div;

use uom::si::f64::Time;

/// Represents the result of dividing a type `T` by [`uom::si::f64::Time`].
///
/// This type alias is commonly used to express the time derivative of a
/// physical quantity. For example, `TimeDerivativeOf<Length> == Velocity`.
pub type TimeDerivativeOf<T> = <T as Div<Time>>::Output;

/// Provides an associated type for the time derivative of `Self`.
///
/// Types implementing this trait can express their rate of change with respect
/// to time via the associated type `TimeDerivative`.
///
/// For example, if `Self` is a [`ThermodynamicTemperature`], then
/// `Self::TimeDerivative` represents the rate of temperature change (dT/dt).
pub trait HasTimeDerivative {
    type TimeDerivative;
}

/// All types implementing `Div<Time>` provide a `TimeDerivative`.
///
/// This implementation enables all `uom::si::Quantity` types to automatically
/// provide the `TimeDerivative` associated type from [`HasTimeDerivative`]
/// without further boilerplate.
impl<T> HasTimeDerivative for T
where
    T: Div<Time>,
{
    type TimeDerivative = TimeDerivativeOf<T>;
}
