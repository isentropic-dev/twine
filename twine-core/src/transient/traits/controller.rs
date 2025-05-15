use std::convert::Infallible;

use crate::{
    transient::{types::TimeIncrement, Integrator, Simulation, StepError, Temporal},
    Component,
};

/// A trait for controlling the advancement of a simulation step.
///
/// A `Controller` manages each iteration of a simulation loop by adjusting the
/// input proposed by an [`Integrator`] before calling the component.
/// This approach enables custom behaviors such as feedback control, constraint
/// enforcement, input validation, or domain-specific preprocessing.
///
/// # Role in Simulation
///
/// A controller is the primary mechanism for progressing a [`Simulation`].
/// Its default [`step`] implementation orchestrates the core simulation loop:
///
/// 1. Request a proposed input from the [`Integrator`].
/// 2. Optionally adjust that input using [`adjust_input`].
/// 3. Call the component with the adjusted input.
/// 4. Record the resulting input/output pair as a new [`TimeStep`].
///
///
/// By separating numerical stepping (via the integrator) from policy-driven
/// control logic (via the controller), this design enables modular and reusable
/// simulation strategies.
///
/// # Typical Usage
///
/// To advance a simulation, call [`step`] on a controller implementation:
///
/// ```ignore
/// controller.step(&mut simulation, &integrator, dt)?;
/// ```
///
/// ## Default Implementation
///
/// The unit type `()` implements [`Controller`] as a no-op pass-through,
/// making it easy to step a simulation when no input adjustment is needed:
///
/// ```ignore
/// ().step(&mut simulation, &integrator, dt)?;
/// ```
pub trait Controller<C>
where
    C: Component,
    C::Input: Clone + Temporal,
{
    /// The error type returned if control logic fails.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Adjusts the proposed input before component evaluation.
    ///
    /// Given a candidate input and the simulation’s current state, this method
    /// applies control logic such as feedback, constraint enforcement, or other
    /// context-specific transformations.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::Error)` if the input is invalid or control logic fails.
    fn adjust_input(
        &self,
        simulation: &Simulation<C>,
        input: C::Input,
    ) -> Result<C::Input, Self::Error>;

    /// Advances the simulation by one time step.
    ///
    /// Refer to the trait documentation for a detailed explanation of the control flow.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if the integrator, controller, or component fails.
    fn step<I>(
        &self,
        simulation: &mut Simulation<C>,
        integrator: &I,
        dt: TimeIncrement,
    ) -> Result<(), StepError<C, I, Self>>
    where
        I: Integrator<C>,
        Self: Sized,
    {
        let proposed = integrator
            .propose_input(simulation, dt)
            .map_err(StepError::Integrator)?;

        let input = self
            .adjust_input(simulation, proposed)
            .map_err(StepError::Controller)?;

        let output = simulation
            .call_component(input.clone())
            .map_err(StepError::Component)?;

        simulation.push_step(input, output);

        Ok(())
    }
}

/// Implements [`Controller`] as a no-op pass-through for the unit type `()`.
///
/// This allows users to step a simulation without any control logic:
///
/// ```ignore
/// ().step(&mut simulation, &integrator, dt)?;
/// ```
///
/// It always returns the proposed input unchanged and never fails.
impl<C> Controller<C> for ()
where
    C: Component,
    C::Input: Clone + Temporal,
{
    type Error = Infallible;

    fn adjust_input(
        &self,
        _simulation: &Simulation<C>,
        input: C::Input,
    ) -> Result<C::Input, Self::Error> {
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::Time,
        time::{minute, second},
    };

    use crate::transient::{integrators::AdvanceTime, test_utils::EchoTime};

    #[test]
    fn unit_controller_steps_simulation() {
        let input = Time::new::<minute>(0.0);
        let mut sim = Simulation::new(EchoTime, input).unwrap();

        let integrator = AdvanceTime;
        let controller = ();

        let dt = TimeIncrement::new::<minute>(1.0).unwrap();
        controller.step(&mut sim, &integrator, dt).unwrap();

        let history = sim.history();
        assert_eq!(history.len(), 2);

        assert_eq!(history[0].input, Time::new::<second>(0.0));
        assert_eq!(history[0].output, Time::new::<second>(0.0));

        assert_eq!(history[1].input, Time::new::<second>(60.0));
        assert_eq!(history[1].output, Time::new::<second>(60.0));
    }
}
