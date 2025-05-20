mod time_increment;

use std::{fmt::Debug, ops::Div};

use uom::si::f64::Time;

use crate::Component;

use super::Temporal;

pub use time_increment::{TimeIncrement, TimeIncrementError};

/// A snapshot of a component’s behavior at a single point in simulation time.
///
/// A `TimeStep` records the input and corresponding output of a component
/// during a single simulation step.
/// It forms the core structure used to track and advance the simulation timeline.
///
/// The struct is generic over the component type `C`.
/// It derives `Debug`, `Clone`, `Copy`, `PartialEq`, and `Default` when
/// applicable to both `C::Input` and `C::Output`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeStep<C>
where
    C: Component,
    C::Input: Temporal,
{
    pub input: C::Input,
    pub output: C::Output,
}

impl<C> TimeStep<C>
where
    C: Component,
    C::Input: Temporal,
{
    /// Constructs a new [`TimeStep`] from the given input and output.
    pub fn new(input: C::Input, output: C::Output) -> Self {
        Self { input, output }
    }
}

/// The type representing the time derivative of `T`.
///
/// Defined as the result of dividing `T` by [`Time`], this alias is commonly
/// used in simulation contexts where `T` is a physical quantity.
///
/// # Examples
///
/// - `TimeDerivativeOf<Length>` = `Velocity`
/// - `TimeDerivativeOf<Velocity>` = `Acceleration`
pub type TimeDerivativeOf<T> = <T as Div<Time>>::Output;
