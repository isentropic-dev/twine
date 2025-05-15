use uom::si::f64::Time;

use super::{Temporal, TimeStep};
use crate::Component;

/// Manages a timeline of simulation steps for a [`Component`].
///
/// A [`Simulation`] records the evolution of a system over time by maintaining
/// a sequence of [`TimeStep`]s, each representing the input and corresponding
/// output of the component at a specific point in simulated time.
///
/// This type supports simulation stepping via a [`Controller`] and [`Integrator`],
/// and exposes utilities to inspect or extend the simulation history.
///
/// [`Controller`]: crate::transient::Controller
/// [`Integrator`]: crate::transient::Integrator
pub struct Simulation<C>
where
    C: Component,
    C::Input: Clone + Temporal,
{
    component: C,
    history: Vec<TimeStep<C>>,
}

impl<C> Simulation<C>
where
    C: Component,
    C::Input: Clone + Temporal,
{
    /// Creates a new simulation from the given component and initial input.
    ///
    /// Evaluates the component once and stores the result as the first step.
    ///
    /// # Errors
    ///
    /// Returns `Err(C::Error)` if the component fails on the initial input.
    pub fn new(component: C, initial_input: C::Input) -> Result<Self, C::Error> {
        let output = component.call(initial_input.clone())?;
        Ok(Self {
            component,
            history: vec![TimeStep::new(initial_input, output)],
        })
    }

    /// Evaluates the component at a given input without modifying simulation state.
    ///
    /// This method is used by [`Controller`]s to evaluate adjusted inputs, and
    /// can also be used by [`Integrator`]s that require mid-step evaluations,
    /// such as when computing intermediate derivatives.
    ///
    /// Internally delegates to `self.component.call(input)`, but exists to
    /// establish a consistent, simulation-aware evaluation boundary.
    ///
    /// # Errors
    ///
    /// Returns an error if the component fails when called with the provided input.
    ///
    /// [`Controller`]: crate::transient::Controller
    /// [`Integrator`]: crate::transient::Integrator
    pub fn call_component(&self, input: C::Input) -> Result<C::Output, C::Error> {
        self.component.call(input)
    }

    /// Appends a new input/output pair to the simulation history.
    ///
    /// # Panics
    ///
    /// Panics if `input.get_time()` is less than or equal to the last recorded
    /// time, ensuring that simulation time always moves forward.
    pub fn push_step(&mut self, input: C::Input, output: C::Output) {
        let t_new = input.get_time();
        let t_prev = self.current_time();

        assert!(
            t_new > t_prev,
            "Step time {t_new:?} must be greater than previous {t_prev:?}"
        );

        self.history.push(TimeStep::new(input, output));
    }

    /// Returns a reference to the most recent step in the simulation.
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

    /// Returns a reference to the component used in the simulation.
    ///
    /// Useful for inspecting configuration or accessing internal fields.
    pub fn component(&self) -> &C {
        &self.component
    }

    /// Returns a reference to the complete simulation history.
    pub fn history(&self) -> &[TimeStep<C>] {
        &self.history
    }

    /// Returns an iterator over all time steps recorded so far.
    pub fn iter_history(&self) -> impl Iterator<Item = &TimeStep<C>> {
        self.history.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{f64::Time, time::second};

    use crate::transient::test_utils::EchoTime;

    #[test]
    fn starts_with_single_step() {
        let input = Time::new::<second>(0.0);
        let sim = Simulation::new(EchoTime, input).unwrap();

        assert_eq!(sim.history().len(), 1);
        assert_eq!(sim.current_time(), input);
        assert_eq!(sim.current_step().output, input);
    }

    #[test]
    fn can_add_step_with_later_time() {
        // Start with time = 0 s.
        let input_0 = Time::new::<second>(0.0);
        let mut sim = Simulation::new(EchoTime, input_0).unwrap();

        // Add step at time = 1 s.
        let input_1 = Time::new::<second>(1.0);
        let output_1 = sim.call_component(input_1).unwrap();
        sim.push_step(input_1, output_1);

        assert_eq!(sim.history().len(), 2);
        assert_eq!(sim.current_time(), input_1);
        assert_eq!(sim.current_step().output, input_1);
    }

    #[test]
    #[should_panic(expected = "Step time")]
    fn panics_on_non_monotonic_time() {
        let input_0 = Time::new::<second>(2.0);
        let mut sim = Simulation::new(EchoTime, input_0).unwrap();

        let input_bad = Time::new::<second>(1.5);
        let output_bad = sim.call_component(input_bad).unwrap();
        sim.push_step(input_bad, output_bad);
    }
}
