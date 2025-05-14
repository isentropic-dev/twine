use std::convert::Infallible;

use uom::si::f64::Time;

use crate::{
    transient::{Integrator, Simulation, StepError, Temporal, TimeStep},
    Component,
};

/// A trait for controlling the advancement of a simulation step.
///
/// A `Controller` manages each iteration of a simulation loop by adjusting the
/// input proposed by an [`Integrator`] before evaluating the component.
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
/// 3. Evaluate the component with the adjusted input.
/// 4. Record the resulting input/output pair as a new [`TimeStep`].
///
/// This design separates numerical stepping (via the integrator) from
/// policy-driven control logic (via the controller), enabling modular and
/// reusable simulation strategies.
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
    /// context-specific transformation.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::Error)` if the input is invalid or control logic fails.
    fn adjust_input(
        &self,
        simulation: &Simulation<C>,
        input: C::Input,
    ) -> Result<C::Input, Self::Error>;

    /// Advances the simulation by one fixed-size time step.
    ///
    /// This method drives the simulation loop and is typically called
    /// externally to progress the system.
    ///
    /// See the trait-level documentation for a detailed description of the control flow.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if the integrator, controller, or component fails.
    fn step<'a, I>(
        &self,
        simulation: &'a mut Simulation<C>,
        integrator: &I,
        dt: Time,
    ) -> Result<&'a TimeStep<C>, StepError<C, I, Self>>
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

        Ok(simulation.current_step())
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
/// It always returns the proposed input unchanged, and never fails.
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
    fn simulation_steps_with_unit_controller() {
        let input = Time::new::<minute>(0.0);
        let mut sim = Simulation::new(EchoTime, input).unwrap();

        let integrator = AdvanceTime;
        let controller = ();

        let dt = Time::new::<minute>(1.0);
        controller.step(&mut sim, &integrator, dt).unwrap();

        let history = sim.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].input, Time::new::<second>(0.0));
        assert_eq!(history[0].output, Time::new::<second>(0.0));
        assert_eq!(history[1].input, Time::new::<second>(60.0));
        assert_eq!(history[1].output, Time::new::<second>(60.0));
    }
}
