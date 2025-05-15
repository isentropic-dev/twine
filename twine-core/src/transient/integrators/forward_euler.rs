use std::{
    convert::Infallible,
    fmt::Debug,
    ops::{Add, Mul},
};

use uom::si::f64::Time;

use crate::transient::{
    HasTimeDerivative, Integrator, Simulation, StatefulComponent, Temporal, TimeIncrement,
};

/// A first-order explicit integrator using the forward Euler method.
///
/// `ForwardEuler` implements the [`Integrator`] trait for [`StatefulComponent`]s
/// by updating state using a time-scaled derivative increment.
///
/// This method is suitable for simple dynamic systems where performance and
/// simplicity are prioritized over numerical accuracy.
#[derive(Debug)]
pub struct ForwardEuler;

impl<C> Integrator<C> for ForwardEuler
where
    C: StatefulComponent,
    C::Input: Clone + Temporal,
    C::State: Add<Output = C::State>,
    <C::State as HasTimeDerivative>::TimeDerivative: Mul<Time, Output = C::State>,
{
    type Error = Infallible;

    /// Computes the next input using forward Euler integration.
    ///
    /// Applies the update rule:
    /// ```text
    ///   state_{n+1} = state_n + derivative_n * dt
    ///   time_{n+1}  = time_n  + dt
    /// ```
    ///
    /// Requires:
    /// - The component’s state supports addition.
    /// - The time derivative, when scaled by `dt`, can be added to the state.
    fn propose_input(
        &self,
        simulation: &Simulation<C>,
        dt: TimeIncrement,
    ) -> Result<C::Input, Self::Error> {
        let current_step = simulation.current_step();
        let current_time = current_step.input.get_time();

        let current_state = C::extract_state(&current_step.input);
        let current_deriv = C::extract_derivative(&current_step.output);

        let new_time = current_time + dt;
        let new_state = current_state + current_deriv * (*dt);

        let next_input = C::apply_state(&current_step.input, new_state).with_time(new_time);

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

    use crate::transient::{
        test_utils::{MovingPoint, PointInput},
        Simulation, TimeIncrement,
    };

    #[test]
    fn forward_euler_advances_state_correctly() {
        // Start with position at 5 meters and time at 10 seconds.
        let sim = Simulation::new(
            MovingPoint::new(Velocity::new::<meter_per_second>(2.0)),
            PointInput {
                position: Length::new::<meter>(5.0),
                time: Time::new::<second>(10.0),
            },
        )
        .unwrap();

        // Step forward by one minute.
        let dt = TimeIncrement::new::<minute>(1.0).unwrap();
        let next_input = ForwardEuler.propose_input(&sim, dt).unwrap();

        // Expect: position = 5 m + 2 m/s * 60 s = 125 m
        //         time = 10 s + 60 s = 70 s
        assert_eq!(next_input.position, Length::new::<meter>(125.0));
        assert_eq!(next_input.time, Time::new::<second>(70.0));
    }
}
