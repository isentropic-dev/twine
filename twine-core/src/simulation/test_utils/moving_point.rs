use std::convert::Infallible;

use uom::si::f64::{Length, Time, Velocity};

use crate::{
    simulation::{HasTime, StatefulComponent},
    Component,
};

/// A test component representing a point moving at constant velocity.
///
/// Simulates a first-order system with a known analytic solution:
///
/// ```text
///   position_{n+1} = position_n + velocity * dt
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MovingPoint {
    pub(crate) velocity: Velocity,
}

/// Input to the `MovingPoint` component, consisting of a position and time.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointInput {
    pub position: Length,
    pub time: Time,
}

/// Output from the `MovingPoint` component, which is always its constant velocity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointOutput {
    pub velocity: Velocity,
}

impl MovingPoint {
    /// Creates a new `MovingPoint` from a `Velocity` value.
    pub(crate) fn new(velocity: Velocity) -> Self {
        Self { velocity }
    }
}

impl HasTime for PointInput {
    fn get_time(&self) -> Time {
        self.time
    }

    fn with_time(mut self, time: Time) -> Self {
        self.time = time;
        self
    }
}

impl Component for MovingPoint {
    type Input = PointInput;
    type Output = PointOutput;
    type Error = Infallible;

    fn call(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(PointOutput {
            velocity: self.velocity,
        })
    }
}

impl StatefulComponent for MovingPoint {
    type State = Length;

    fn extract_state(input: &Self::Input) -> Self::State {
        input.position
    }

    fn extract_derivative(output: &Self::Output) -> Velocity {
        output.velocity
    }

    fn apply_state(input: &Self::Input, state: Self::State) -> Self::Input {
        PointInput {
            position: state,
            time: input.time,
        }
    }
}
