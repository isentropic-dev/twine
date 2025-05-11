use std::{
    fmt::Debug,
    ops::{Add, Mul},
};

use uom::si::f64::Time;

use crate::transient::{HasTime, HasTimeDerivative, StateIntegrator, StatefulComponent, TimeStep};

/// An explicit, first-order forward Euler time integrator.
///
/// This integrator advances a component’s state over time using the simple
/// forward Euler formula:
///
/// ```text
///   state_{n+1} = state_n + derivative_n * dt
///   time_{n+1}  = time_n  + dt
/// ```
///
/// It is compatible with any [`StatefulComponent`] whose `State`:
/// - Supports addition with itself, allowing `state + delta_state`.
/// - Has an associated time derivative type that supports scaling by a time
///   step, allowing `derivative * dt`.
///
/// In trait terms:
/// - `State: Add<Output = State>`
/// - `<State as HasTimeDerivative>::TimeDerivative: Mul<Time, Output = State>`
///
/// These trait bounds enable forward propagation of a component’s state by
/// applying a time-scaled derivative increment at each step.
pub struct ForwardEuler;

impl<C> StateIntegrator<C> for ForwardEuler
where
    C: StatefulComponent,
    C::Input: Clone + Debug + HasTime,
    C::Output: Clone + Debug,
    C::State: Add<Output = C::State>,
    <C::State as HasTimeDerivative>::TimeDerivative: Mul<Time, Output = C::State>,
{
    /// Advances the component state by one step using the forward Euler method.
    fn integrate_state(
        &self,
        _component: &C,
        current: &TimeStep<C>,
        dt: Time,
    ) -> Result<C::Input, C::Error> {
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

    use crate::{
        transient::test_utils::{MovingPoint, PointInput},
        Component,
    };

    #[test]
    fn take_a_forward_euler_step() {
        let integrator = ForwardEuler;
        let component = MovingPoint::new(Velocity::new::<meter_per_second>(2.0));

        // Start with position at 5 meters and time at 10 seconds.
        let input = PointInput {
            position: Length::new::<meter>(5.0),
            time: Time::new::<second>(10.0),
        };
        let output = component.call(input).unwrap();
        let current_step = TimeStep { input, output };

        // Step forward by one minute.
        let first_step_input = integrator
            .integrate_state(&component, &current_step, Time::new::<minute>(1.0))
            .unwrap();

        assert_eq!(
            first_step_input,
            PointInput {
                position: Length::new::<meter>(125.0),
                time: Time::new::<second>(70.0),
            },
        );
    }
}
