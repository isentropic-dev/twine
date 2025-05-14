use std::{convert::Infallible, fmt::Debug};

use uom::si::f64::Time;

use crate::{
    transient::{Integrator, Temporal, TimeStep},
    Component,
};

/// An integrator that advances simulation time without modifying state.
///
/// `AdvanceTime` is a minimal implementation of the [`Integrator`] trait that
/// increments the simulation time in the component’s input while leaving all
/// other fields unchanged.
/// It is useful for time-driven components that do not depend on internal state.
#[derive(Debug)]
pub struct AdvanceTime;

impl<C> Integrator<C> for AdvanceTime
where
    C: Component,
    C::Input: Clone + Temporal,
{
    type Error = Infallible;

    /// Computes the next input by incrementing the simulation time.
    ///
    /// This integrator advances the input’s time by `dt` and returns the result.
    /// The rest of the input is unchanged.
    ///
    /// # Panics
    ///
    /// Panics if `history` is empty, which indicates incorrect integrator use.
    fn propose_input(
        &self,
        _component: &C,
        history: &[TimeStep<C>],
        dt: Time,
    ) -> Result<C::Input, Self::Error> {
        let current = history
            .last()
            .expect("Simulation history must be non-empty");

        let current_time = current.input.get_time();
        let new_time = current_time + dt;
        let next_input = current.input.clone().with_time(new_time);

        Ok(next_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::Time,
        time::{hour, minute},
    };

    use crate::Component;

    /// A test component that echoes the current simulation time.
    struct EchoTime;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Input {
        time: Time,
    }

    impl Temporal for Input {
        fn get_time(&self) -> Time {
            self.time
        }

        fn with_time(self, time: Time) -> Self {
            Self { time }
        }
    }

    impl From<Time> for Input {
        fn from(time: Time) -> Self {
            Self { time }
        }
    }

    impl Component for EchoTime {
        type Input = Input;
        type Output = Time;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(input.time)
        }
    }

    #[test]
    fn advance_time_increments_simulation_time() {
        // Start with time at 1 hour.
        let input = Time::new::<hour>(1.0).into();
        let output = EchoTime.call(input).unwrap();
        let history = vec![TimeStep { input, output }];

        // Step forward by 5 minutes.
        let next_step_input = AdvanceTime
            .propose_input(&EchoTime, &history, Time::new::<minute>(5.0))
            .unwrap();

        assert_eq!(next_step_input.time, Time::new::<minute>(65.0));
    }
}
