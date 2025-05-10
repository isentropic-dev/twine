pub mod integrators;
mod traits;

#[cfg(test)]
mod test_utils;

use std::fmt::Debug;

use uom::si::f64::Time;

pub use traits::{HasTime, StateIntegrator, StatefulComponent};

/// A snapshot of a [`StatefulComponent`]’s result at a single simulation step.
///
/// A `TimeStep` records the `input` and corresponding `output` of a component
/// at a specific point in simulated time.
/// It represents an atomic step in the simulation’s progression and acts as a
/// fundamental unit for history tracking, stepping, and integration.
#[derive(Clone, Debug)]
pub struct TimeStep<C>
where
    C: StatefulComponent,
    C::Input: Clone + Debug,
    C::Output: Clone + Debug,
{
    pub input: C::Input,
    pub output: C::Output,
}

/// A `Simulation` evolves a [`StatefulComponent`] over time using a [`StateIntegrator`].
///
/// It holds a component, an integrator, and a timeline of [`TimeStep`]s
/// representing the system’s evolution.
/// Each call to [`step()`] advances the simulation by integrating the
/// component’s state over a time increment, then appending the resulting
/// [`TimeStep`] to the simulation history.
pub struct Simulation<C, I>
where
    C: StatefulComponent,
    C::Input: Clone + Debug + HasTime,
    C::Output: Clone + Debug,
    I: StateIntegrator<C>,
{
    component: C,
    integrator: I,
    history: Vec<TimeStep<C>>,
}

impl<C, I> Simulation<C, I>
where
    C: StatefulComponent,
    C::Input: Clone + Debug + HasTime,
    C::Output: Clone + Debug,
    I: StateIntegrator<C>,
{
    /// Initializes the simulation with a component, integrator, and initial input.
    ///
    /// The initial output is computed immediately and recorded as the first
    /// [`TimeStep`] in the simulation’s history.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component fails on the initial input.
    pub fn new(component: C, integrator: I, initial_input: C::Input) -> Result<Self, C::Error> {
        let initial_output = component.call(initial_input.clone())?;
        Ok(Self {
            component,
            integrator,
            history: vec![TimeStep {
                input: initial_input,
                output: initial_output,
            }],
        })
    }

    /// Advances the simulation forward by one time step of size `dt`.
    ///
    /// The component’s current state is evolved using the [`StateIntegrator`],
    /// and the resulting [`TimeStep`] is appended to the simulation history.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component or integrator fails.
    ///
    /// # Panics
    ///
    /// Panics if the simulation history is unexpectedly empty, which should be
    /// impossible after successful initialization via [`Simulation::new`].
    pub fn step(&mut self, dt: Time) -> Result<(), C::Error> {
        let last = self.history.last().unwrap();
        let next = self.integrator.step(&self.component, last, dt)?;
        self.history.push(next);
        Ok(())
    }

    /// Returns the full history of [`TimeStep`]s in the simulation.
    pub fn history(&self) -> &[TimeStep<C>] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::{Length, Time, Velocity},
        length::meter,
        time::second,
        velocity::meter_per_second,
    };

    use integrators::ForwardEuler;
    use test_utils::{MovingPoint, PointInput};

    #[test]
    fn moving_point_moves_as_expected() {
        let mut sim = Simulation::new(
            // A component with constant velocity: 3 m/s.
            MovingPoint::new(Velocity::new::<meter_per_second>(3.0)),
            // Use the forward Euler integration method.
            ForwardEuler,
            // Initial state: position = 0 m, time = 0 s.
            PointInput::default(),
        )
        .unwrap();

        // Step forward by 1 second.
        sim.step(Time::new::<second>(1.0)).unwrap();

        // Then step forward by 4 more seconds.
        sim.step(Time::new::<second>(4.0)).unwrap();

        // Expect 3 recorded time steps: initial + 2 steps.
        assert_eq!(sim.history().len(), 3);

        // Collect simulation timestamps and positions.
        let times: Vec<_> = sim.history().iter().map(|step| step.input.time).collect();
        let positions: Vec<_> = sim
            .history()
            .iter()
            .map(|step| step.input.position)
            .collect();

        assert_eq!(
            times,
            vec![
                Time::new::<second>(0.0),
                Time::new::<second>(1.0),
                Time::new::<second>(5.0),
            ]
        );

        assert_eq!(
            positions,
            vec![
                Length::new::<meter>(0.0),
                Length::new::<meter>(3.0),
                Length::new::<meter>(15.0),
            ]
        );
    }
}
