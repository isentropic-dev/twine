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

impl Temporal for Time {
    fn get_time(&self) -> Time {
        *self
    }

    fn with_time(self, time: Time) -> Self {
        time
    }
}
