use std::{
    convert::Infallible,
    fmt::Debug,
    ops::{Add, Mul},
};

use uom::si::f64::Time;

use crate::transient::{HasTimeDerivative, Integrator, StatefulComponent, Temporal, TimeStep};

/// A first-order explicit integrator using the forward Euler method.
///
/// Advances a [`StatefulComponent`] by applying a time-scaled derivative
/// increment to its state.
#[derive(Debug)]
pub struct ForwardEuler;

impl<C> Integrator<C> for ForwardEuler
where
    C: StatefulComponent,
    C::Input: Temporal,
    C::State: Add<Output = C::State>,
    <C::State as HasTimeDerivative>::TimeDerivative: Mul<Time, Output = C::State>,
{
    type Error = Infallible;

    /// Computes the next input using forward Euler integration.
    ///
    /// Applies the update rule:
    ///
    /// ```text
    ///   state_{n+1} = state_n + derivative_n * dt
    ///   time_{n+1}  = time_n  + dt
    /// ```
    ///
    /// Requires:
    /// - `State: Add<Output = State>`
    /// - `<State as HasTimeDerivative>::TimeDerivative: Mul<Time, Output = State>`
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
        let current_state = C::extract_state(&current.input);
        let current_deriv = C::extract_derivative(&current.output);

        let new_time = current_time + dt;
        let new_state = current_state + current_deriv * dt;

        let next_input = C::apply_state(&current.input, new_state).with_time(new_time);

        Ok(next_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::{Length, Time, Velocity},
        length::meter,
        time::{minute, second},
        velocity::meter_per_second,
    };

    use crate::transient::test_utils::MovingPoint;

    #[test]
    fn forward_euler_advances_state_correctly() {
        let component = MovingPoint::new(Velocity::new::<meter_per_second>(2.0));

        // Start with position at 5 meters and time at 10 seconds.
        let position = Length::new::<meter>(5.0);
        let time = Time::new::<second>(10.0);
        let history = component.initial_history_at(position, time);

        // Step forward by one minute.
        let next_step_input = ForwardEuler
            .propose_input(&component, &history, Time::new::<minute>(1.0))
            .unwrap();

        // Expect position and time to be updated accordingly.
        assert_eq!(next_step_input.position, Length::new::<meter>(125.0));
        assert_eq!(next_step_input.time, Time::new::<second>(70.0));
    }
}
