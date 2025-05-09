mod traits;

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
