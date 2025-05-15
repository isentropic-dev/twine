use std::ops::Div;

use uom::si::f64::Time;

/// A trait for types that represent a point in simulation time.
///
/// Types that implement `Temporal` carry an internal timestamp and can produce
/// a modified version with a different time.
///
/// This trait is typically implemented by component input types so they can
/// participate in time-based integration and control.
///
/// # Example
///
/// ```
/// use twine_core::transient::Temporal;
/// use uom::si::f64::Time;
///
/// struct MyInput {
///     time: Time,
///     value: f64,
/// }
///
/// impl Temporal for MyInput {
///     fn get_time(&self) -> Time {
///         self.time
///     }
///
///     fn with_time(self, time: Time) -> Self {
///         Self { time, ..self }
///     }
/// }
/// ```
pub trait Temporal: Sized {
    /// Returns the current simulation time.
    fn get_time(&self) -> Time;

    /// Returns a new instance with the specified simulation time.
    #[must_use]
    fn with_time(self, time: Time) -> Self;
}

/// A trait indicating that a type has a time derivative.
///
/// This trait defines the type representing the rate of change of `Self` with
/// respect to time.
/// It is used to describe the evolution of state in dynamic simulations by
/// relating physical quantities to their derivatives.
/// For example, `Length` has a time derivative of `Velocity`.
pub trait HasTimeDerivative {
    /// The result of dividing `Self` by [`Time`].
    type TimeDerivative;
}

/// Automatically implemented for types that support division by [`Time`].
///
/// This blanket implementation covers most physical quantities in `uom`,
/// allowing them to be used seamlessly in simulations.
///
/// # Example
///
/// `Mass` implements `Div<Time>`, so `HasTimeDerivative` is automatically
/// satisfied with `TimeDerivative = MassRate`.
impl<T> HasTimeDerivative for T
where
    T: Div<Time>,
{
    type TimeDerivative = T::Output;
}

impl Temporal for Time {
    fn get_time(&self) -> Time {
        *self
    }

    fn with_time(self, time: Time) -> Self {
        time
    }
}
