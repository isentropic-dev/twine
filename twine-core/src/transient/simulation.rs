use thiserror::Error;
use uom::si::f64::Time;

use crate::Component;

use super::{Controller, Integrator, Temporal, TimeIncrement, TimeIncrementError, TimeStep};

/// Manages the simulation of a dynamic [`Component`] over time.
///
/// A [`Simulation`] owns a [`Component`] and a history of [`TimeStep`]s that
/// record its evolution across discrete time steps.
/// At each step, it uses an [`Integrator`] to propose the next input and a
/// [`Controller`] to optionally adjust it before evaluation.
pub struct Simulation<C>
where
    C: Component,
    C::Input: Temporal,
{
    component: C,
    history: Vec<TimeStep<C>>,
}

/// Error type for failures that can occur during a simulation step.
///
/// This error groups failures from any stage of the step process:
///
/// - [`Integrator`]: Failed to generate a proposed input
/// - [`Controller`]: Failed to adjust the proposed input
/// - [`Component`]: Failed during evaluation
///
/// Returned by [`Simulation::step`].
#[derive(Debug, Error)]
pub enum StepError<C, I, K>
where
    C: Component,
    C::Input: Temporal,
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

/// Defines how the simulation advances over time.
///
/// A `Stepping` value specifies the policy used by [`Simulation::advance`] to
/// determine the size and number of time steps.
/// Each variant defines a different rule for advancing simulation time.
///
/// See [`Simulation::advance`] for details.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stepping {
    /// Advance by a fixed `dt` for `num_steps`.
    FixedSteps { dt: TimeIncrement, num_steps: usize },

    /// Advance with a fixed `dt` until `end_time` (may overstep).
    UntilTime { dt: TimeIncrement, end_time: Time },

    /// Advance in `num_steps` of equal size to reach exactly `end_time`.
    StepsToTime { num_steps: usize, end_time: Time },
}

impl<C> Simulation<C>
where
    C: Component,
    C::Input: Temporal,
{
    /// Creates a new simulation from a component and an initial input.
    ///
    /// Evaluates the component once with the initial input, storing the result
    /// as the first [`TimeStep`] in the simulation history.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component fails to evaluate the initial input.
    pub fn new(component: C, initial_input: C::Input) -> Result<Self, C::Error>
    where
        C::Input: Clone,
    {
        let output = component.call(initial_input.clone())?;
        Ok(Self {
            component,
            history: vec![TimeStep::new(initial_input, output)],
        })
    }

    /// Advances the simulation by a single time increment.
    ///
    /// Performs one full simulation step:
    ///
    /// 1. Proposes the next input using the [`Integrator`].
    /// 2. Adjusts the proposed input via the [`Controller`].
    /// 3. Evaluates the [`Component`] with the adjusted input.
    /// 4. Records the result as a new [`TimeStep`] in the history.
    ///
    /// # Parameters
    ///
    /// - `dt`: The time increment to advance by.
    /// - `integrator`: Proposes the next input based on the current state.
    /// - `controller`: Adjusts the proposed input before evaluation.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if any part of the step process fails.
    pub fn step<I, K>(
        &mut self,
        dt: TimeIncrement,
        integrator: &I,
        controller: &K,
    ) -> Result<(), StepError<C, I, K>>
    where
        C::Input: Clone,
        I: Integrator<C>,
        K: Controller<C>,
    {
        let proposed = integrator
            .propose_input(self, dt)
            .map_err(StepError::Integrator)?;

        let input = controller
            .adjust_input(self, proposed)
            .map_err(StepError::Controller)?;

        let output = self
            .component
            .call(input.clone())
            .map_err(StepError::Component)?;

        self.history.push(TimeStep::new(input, output));

        Ok(())
    }

    /// Advances the simulation according to a specified [`Stepping`] policy.
    ///
    /// This method repeatedly calls [`step`] to simulate the evolution of a
    /// [`Component`] over time.
    /// Each step advances the simulation by one time increment:
    ///
    /// - Proposes the next input using the [`Integrator`],
    /// - Adjusts it via the [`Controller`],
    /// - Evaluates the [`Component`],
    /// - Records the result as a new [`TimeStep`] in the simulation history.
    ///
    /// # Stepping Policies
    ///
    /// - [`Stepping::FixedSteps`]:
    ///   Advances using a fixed time increment `dt` for `num_steps`.
    ///
    /// - [`Stepping::UntilTime`]:
    ///   Repeatedly steps forward by `dt` until reaching or exceeding `end_time`.
    ///
    /// - [`Stepping::StepsToTime`]:
    ///   Divides the interval from the current time to `end_time` into
    ///   `num_steps` equal intervals.
    ///
    /// # Parameters
    ///
    /// - `stepping`: The policy that determines how time advances.
    /// - `integrator`: Proposes a new input at each step.
    /// - `controller`: Adjusts the proposed input before evaluation.
    ///
    /// # Returns
    ///
    /// - `Ok(Self)`: The simulation after advancing through all steps.
    /// - `Err(StepError)`: If any step fails due to an error in the integrator,
    ///   controller, or the component.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if any step in the sequence fails.
    ///
    /// # Panics
    ///
    /// Panics if the stepping policy is invalid, such as having zero steps or
    /// an end time that is not after the current simulation time.
    ///
    /// # Examples
    ///
    /// A typical pattern is to create a simulation and immediately advance it:
    ///
    /// ```ignore
    /// let component = MyComponent::new(/* ... */);
    /// let initial_input = MyInput::new(/* ... */);
    /// let stepping = Stepping::FixedSteps {
    ///     dt: TimeIncrement::new::<second>(1.0).unwrap(),
    ///     num_steps: 10,
    /// };
    ///
    /// let sim = Simulation::new(component, initial_input)?
    ///     .advance(stepping, &ForwardEuler, &SomeController)?;
    /// ```
    ///
    /// You can also chain multiple `advance` calls.
    /// This is useful when a different integration scheme or control strategy
    /// is needed later in the simulation:
    ///
    /// ```ignore
    /// let sim = Simulation::new(component, initial_input)?
    ///     // Use a simple open-loop integrator early on.
    ///     .advance(Stepping::UntilTime {
    ///         dt: TimeIncrement::new::<minute>(1.0).unwrap(),
    ///         end_time: warmup_end,
    ///     }, &ForwardEuler, &PassThrough)?
    ///     // Then switch to a more accurate integrator and add a controller.
    ///     .advance(Stepping::StepsToTime {
    ///         num_steps: 100,
    ///         end_time: final_time,
    ///     }, &RungeKutta4, &FeedbackController)?;
    /// ```
    pub fn advance<I, K>(
        mut self,
        stepping: Stepping,
        integrator: &I,
        controller: &K,
    ) -> Result<Self, StepError<C, I, K>>
    where
        C::Input: Clone,
        I: Integrator<C>,
        K: Controller<C>,
    {
        let (dt, num_steps) = match stepping {
            Stepping::FixedSteps { dt, num_steps } => {
                assert!(num_steps != 0, "Number of steps cannot be zero");
                (dt, num_steps)
            }

            Stepping::UntilTime { dt, end_time } => {
                let total_interval = self
                    .time_interval_to(end_time)
                    .expect("End time must be after current time");

                let num_steps = total_interval.steps_required(dt);

                (dt, num_steps)
            }

            Stepping::StepsToTime {
                num_steps,
                end_time,
            } => {
                assert!(num_steps != 0, "Number of steps cannot be zero");

                let total_interval = self
                    .time_interval_to(end_time)
                    .expect("End time must be after current time");

                let dt = total_interval / num_steps;

                (dt, num_steps)
            }
        };

        for _ in 0..num_steps {
            self.step(dt, integrator, controller)?;
        }

        Ok(self)
    }

    /// Evaluates the component at a given input without modifying the simulation history.
    ///
    /// Useful for previewing system behavior or computing hypothetical outputs.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component fails to evaluate the input.
    pub fn call_component(&self, input: C::Input) -> Result<C::Output, C::Error> {
        self.component.call(input)
    }

    /// Returns the most recent step in the simulation.
    #[allow(clippy::missing_panics_doc)]
    pub fn current_step(&self) -> &TimeStep<C> {
        self.history
            .last()
            .expect("Simulation history is never empty")
    }

    /// Returns the simulation time of the most recent step.
    pub fn current_time(&self) -> Time {
        self.current_step().input.get_time()
    }

    /// Returns a reference to the simulation’s component.
    pub fn component(&self) -> &C {
        &self.component
    }

    /// Returns a slice of all recorded simulation steps.
    pub fn history(&self) -> &[TimeStep<C>] {
        &self.history
    }

    /// Returns an iterator over all recorded simulation steps.
    pub fn iter_history(&self) -> impl Iterator<Item = &TimeStep<C>> {
        self.history.iter()
    }

    /// Computes the `TimeIncrement` from the current time to the target.
    fn time_interval_to(&self, target: Time) -> Result<TimeIncrement, TimeIncrementError> {
        let current = self.current_time();
        TimeIncrement::from_time(target - current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::Time,
        time::{minute, second},
    };

    use crate::transient::{
        controllers::PassThrough, integrators::AdvanceTime, test_utils::EchoTime,
    };

    #[test]
    fn starts_with_single_step() {
        let input = Time::new::<second>(0.0);
        let sim = Simulation::new(EchoTime, input).unwrap();

        assert_eq!(sim.history().len(), 1);
        assert_eq!(sim.current_time(), input);
        assert_eq!(sim.current_step().output, input);
    }

    #[test]
    fn take_a_single_step() {
        let component = EchoTime;
        let input = Time::new::<minute>(0.0);
        let mut sim = Simulation::new(component, input).unwrap();

        sim.step(
            TimeIncrement::new::<minute>(1.0).unwrap(),
            &AdvanceTime,
            &PassThrough,
        )
        .unwrap();

        let history = sim.history();
        assert_eq!(history.len(), 2);

        assert_eq!(history[0].input, Time::new::<second>(0.0));
        assert_eq!(history[0].output, Time::new::<second>(0.0));

        assert_eq!(history[1].input, Time::new::<second>(60.0));
        assert_eq!(history[1].output, Time::new::<second>(60.0));
    }

    #[test]
    fn advance_with_fixed_steps_works() {
        let input = Time::new::<second>(0.0);
        let sim = Simulation::new(EchoTime, input).unwrap();

        let dt = TimeIncrement::new::<second>(1.0).unwrap();
        let sim = sim
            .advance(
                Stepping::FixedSteps { dt, num_steps: 5 },
                &AdvanceTime,
                &PassThrough,
            )
            .unwrap();

        assert_eq!(sim.history().len(), 6, "1 initial + 5 steps");
        assert_eq!(sim.current_time(), Time::new::<second>(5.0));
    }

    #[test]
    fn advance_until_time_covers_target() {
        let input = Time::new::<second>(0.0);
        let sim = Simulation::new(EchoTime, input).unwrap();

        let dt = TimeIncrement::new::<second>(2.0).unwrap();
        let end_time = Time::new::<second>(7.0);

        let sim = sim
            .advance(
                Stepping::UntilTime { dt, end_time },
                &AdvanceTime,
                &PassThrough,
            )
            .unwrap();

        assert_eq!(sim.history().len(), 5, "1 initial + 4 steps");
        assert!(sim.current_time() >= end_time);
    }

    #[test]
    fn advance_steps_to_time_reaches_target() {
        let input = Time::new::<minute>(0.0);
        let sim = Simulation::new(EchoTime, input).unwrap();

        let end_time = Time::new::<minute>(6.0);

        let sim = sim
            .advance(
                Stepping::StepsToTime {
                    num_steps: 3,
                    end_time,
                },
                &AdvanceTime,
                &PassThrough,
            )
            .unwrap();

        assert_eq!(sim.history().len(), 4, "1 initial + 3 steps");
        assert_eq!(sim.current_time(), end_time);
    }
}
