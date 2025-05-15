use std::convert::Infallible;

use uom::si::f64::{Length, Time, Velocity};

use crate::{
    transient::{StatefulComponent, Temporal},
    Component,
};

/// A test component modeling a point moving at constant velocity.
///
/// This first-order system evolves linearly over time according to:
///
/// ```text
///   position_{n+1} = position_n + velocity * dt
/// ```
///
/// Useful for validating integrators with a predictable linear trajectory.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct MovingPoint {
    pub(crate) velocity: Velocity,
}

/// Input to the `MovingPoint` component, including position and time.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct PointInput {
    pub(crate) position: Length,
    pub(crate) time: Time,
}

/// Output from the `MovingPoint` component, exposing its constant velocity.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct PointOutput {
    pub(crate) velocity: Velocity,
}

impl MovingPoint {
    /// Constructs a new `MovingPoint` with the specified velocity.
    pub(crate) fn new(velocity: Velocity) -> Self {
        Self { velocity }
    }
}

impl Temporal for PointInput {
    fn get_time(&self) -> Time {
        self.time
    }

    fn with_time(self, time: Time) -> Self {
        Self { time, ..self }
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
