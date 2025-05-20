use thiserror::Error;
use uom::si::f64::Time;

use crate::Component;

use super::{Controller, Integrator, Temporal, TimeIncrement, TimeStep};

/// Manages the simulation of a dynamic [`Component`] over time.
///
/// A [`Simulation`] owns a [`Component`] and a history of [`TimeStep`]s that
/// record its evolution across discrete time steps.
/// At each step, it uses an [`Integrator`] to propose the next input and a
/// [`Controller`] to optionally adjust it before evaluation.
pub struct Simulation<C>
where
    C: Component,
    C::Input: Clone + Temporal,
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

impl<C> Simulation<C>
where
    C: Component,
    C::Input: Clone + Temporal,
{
    /// Creates a new simulation from a component and an initial input.
    ///
    /// Evaluates the component once with the initial input, storing the result
    /// as the first [`TimeStep`] in the simulation history.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component fails to evaluate the initial input.
    pub fn new(component: C, initial_input: C::Input) -> Result<Self, C::Error> {
        let output = component.call(initial_input.clone())?;
        Ok(Self {
            component,
            history: vec![TimeStep::new(initial_input, output)],
        })
    }

    /// Advances the simulation by a single time increment.
    ///
    /// Performs a full simulation step:
    ///
    /// 1. Uses the [`Integrator`] to propose the next input.
    /// 2. Adjusts the input with the [`Controller`].
    /// 3. Evaluates the component using the adjusted input.
    /// 4. Records the result as a new [`TimeStep`] in the history.
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
        I: Integrator<C>,
        K: Controller<C>,
        Self: Sized,
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
}
