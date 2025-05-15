mod time_increment;

use std::{fmt::Debug, ops::Div};

use thiserror::Error;
use uom::si::f64::Time;

use crate::Component;

use super::{Controller, Integrator, Temporal};

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

/// An error that can occur when advancing a simulation step using a controller.
///
/// This type aggregates failures from multiple sources involved in progressing
/// the simulation:
///
/// - [`Integrator`]: Failed to generate a candidate input from history.
/// - [`Controller`]: Failed to adjust the proposed input.
/// - [`Component`]: Failed during evaluation on the final input.
///
/// It is returned by [`Controller::step`] and provides granular diagnostics for
/// debugging simulation progression.
#[derive(Debug, Error)]
pub enum StepError<C, I, K>
where
    C: Component,
    C::Input: Clone + Temporal,
    I: Integrator<C>,
    K: Controller<C>,
{
    #[error("Component failed: {0}")]
    Component(C::Error),
    #[error("Controller failed: {0}")]
    Controller(K::Error),
    #[error("Integrator failed: {0}")]
    Integrator(I::Error),
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
