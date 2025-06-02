use std::{iter::FusedIterator, time::Duration};

use thiserror::Error;

use crate::{Component, Integrator};

/// Combines a model and integrator to define a time-stepping simulation.
///
/// A `Simulation` evolves a system's state by integrating changes in input
/// variables and recomputing outputs through a model.
/// It provides a generic and extensible interface for building time-based
/// simulations across multiple domains.
///
/// # Associated Types
///
/// - [`Model`]: A [`Component`] that maps independent variables (`Input`) to
///   dependent variables (`Output`).
/// - [`Integrator`]: Advances selected input variables over time, typically
///   using a numerical scheme such as Forward Euler or Runge-Kutta.
///
/// # State Representation
///
/// A [`State`] captures the complete state of the system at a specific point
/// in time, represented as a consistent pair of `Model::Input` (independent)
/// and `Model::Output` (dependent) values.
///
/// Simulations operate by transforming one state into the next, advancing time
/// through integration and reevaluating outputs via the model.
///
/// # Mapping Methods
///
/// These methods define how a simulation constructs the inputs required by its
/// integrator and model.
/// Besides providing the necessary mapping between simulation state and input
/// types, implementations can also inject context such as historical state,
/// simulation parameters, or input from the outside world.
///
/// - [`prepare_integrator_input`]: Builds the integrator input from the current
///   state, selecting and transforming values needed for integration.
/// - [`prepare_model_input`]: Constructs the next model input using the
///   previous state, the integrator's output, and the effective time step.
///
/// These functions define the boundary between domain-specific modeling and
/// integrator-specific stepping, enabling composable and flexible simulations.
///
/// # Stepping Methods
///
/// The `Simulation` trait provides these methods for advancing the simulation:
///
/// - [`step`]: Advances the simulation by one step from an initial input.
/// - [`step_from_state`]: Advances the simulation from a full known state.
/// - [`step_iter`]: Returns an iterator over simulation steps.
/// - [`step_many`]: Runs multiple steps and collects all resulting states.
pub trait Simulation: Sized {
    type Model: Component;
    type Integrator: Integrator;

    /// Returns a reference to the underlying model component.
    ///
    /// The model maps input (independent) variables to output (dependent)
    /// variables, defining system behavior at a specific point in time.
    fn model(&self) -> &Self::Model;

    /// Returns a reference to the underlying integrator.
    ///
    /// The integrator advances selected input variables across a time step,
    /// typically using a numerical method such as Forward Euler or Runge-Kutta.
    fn integrator(&self) -> &Self::Integrator;

    /// Prepares the integrator input from the current simulation state.
    ///
    /// Extracts and transforms values from the given [`State`], including both
    /// input (independent) and output (dependent) variables, into the form
    /// expected by the [`Integrator`].
    ///
    /// The integrator input may also include derived quantities, closures
    /// for deferred model evaluation, or simulation-specific context such as
    /// history or configuration.
    fn prepare_integrator_input(
        &self,
        state: &State<Self::Model>,
    ) -> <Self::Integrator as Integrator>::Input;

    /// Prepares the model input for the next simulation step.
    ///
    /// Extracts and transforms data from the previous [`State`],
    /// the integrator's output, and the actual time step into a new
    /// [`Component::Input`] for the model.
    ///
    /// The resulting input may update time, advance state variables, or
    /// incorporate simulation-specific behavior such as constraint enforcement,
    /// historical context, or responses to external input.
    ///
    /// # Parameters
    ///
    /// - `prev_state`: The previous [`State`], including both input and output.
    /// - `integrator_output`: The output produced by the integrator.
    /// - `actual_dt`: The time step that was actually taken.
    ///
    /// # Returns
    ///
    /// The next [`Component::Input`] to be passed to the model.
    fn prepare_model_input(
        &self,
        prev_state: &State<Self::Model>,
        integrator_output: <Self::Integrator as Integrator>::Output,
        actual_dt: Duration,
    ) -> <Self::Model as Component>::Input;

    /// Advances the simulation by one time step from an initial input.
    ///
    /// This method first evaluates the model to compute the initial output,
    /// forming a complete [`State`] before delegating to [`step_from_state`].
    /// As a result, the model is called twice: once to initialize the state,
    /// and once after stepping.
    ///
    /// # Parameters
    ///
    /// - `input`: The [`Model::Input`] at the beginning of the step.
    /// - `dt`: The requested time step duration.
    ///
    /// # Returns
    ///
    /// The resulting [`State`] after the simulation step.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if either the model or the integrator fails.
    fn step(
        &self,
        input: <Self::Model as Component>::Input,
        dt: Duration,
    ) -> Result<State<Self::Model>, StepError<Self>>
    where
        <Self::Model as Component>::Input: Clone,
    {
        let output = self.model().call(input.clone()).map_err(StepError::Model)?;
        let state = State { input, output };
        self.step_from_state(&state, dt)
    }

    /// Advances the simulation by one time step from a full system state.
    ///
    /// This method prepares the integrator input from the given [`State`],
    /// applies integration to advance the input, and then evaluates the model
    /// to produce the next output.
    ///
    /// # Parameters
    ///
    /// - `state`: The current [`State`], containing both input and output.
    /// - `dt`: The requested time step duration.
    ///
    /// # Returns
    ///
    /// The resulting [`State`] after the simulation step.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if either the model or the integrator fails.
    fn step_from_state(
        &self,
        state: &State<Self::Model>,
        dt: Duration,
    ) -> Result<State<Self::Model>, StepError<Self>>
    where
        <Self::Model as Component>::Input: Clone,
    {
        let integrator_input = self.prepare_integrator_input(state);

        let (integrator_output, actual_dt) = self
            .integrator()
            .integrate(integrator_input, dt)
            .map_err(StepError::Integrator)?;

        let next_input = self.prepare_model_input(state, integrator_output, actual_dt);

        let next_output = self
            .model()
            .call(next_input.clone())
            .map_err(StepError::Model)?;

        Ok(State {
            input: next_input,
            output: next_output,
        })
    }

    /// Creates an iterator that steps the simulation from an initial input.
    ///
    /// The iterator first yields the input paired with its computed output,
    /// then produces a new [`State`] on each step by advancing time and
    /// reevaluating the model.
    ///
    /// This method is suitable for lazy or streaming evaluation.
    /// It is memory-efficient and supports standard combinators such as
    /// `.take(n)`, `.map(...)`, or `.find(...)`.
    ///
    /// # Parameters
    ///
    /// - `initial_input`: The starting [`Model::Input`] for the simulation.
    /// - `dt`: The requested time step used for each simulation step.
    ///
    /// # Returns
    ///
    /// An iterator yielding `Result<State<Model>, StepError>` values.
    /// On error, the iterator yields the error and terminates.
    fn step_iter(
        &self,
        initial_input: <Self::Model as Component>::Input,
        dt: Duration,
    ) -> impl Iterator<Item = Result<State<Self::Model>, StepError<Self>>>
    where
        <Self::Model as Component>::Input: Clone,
        <Self::Model as Component>::Output: Clone,
    {
        StepIter {
            sim: self,
            input: Some(initial_input),
            output: None,
            dt,
        }
    }

    /// Runs the simulation for a fixed number of steps.
    ///
    /// This method returns `steps + 1` states: the initial input paired with its
    /// computed output, followed by each successive step.
    ///
    /// # Parameters
    ///
    /// - `initial_input`: The starting [`Model::Input`] for the simulation.
    /// - `steps`: The number of steps to perform.
    /// - `dt`: The requested time step used for each simulation step.
    ///
    /// # Returns
    ///
    /// A `Vec<State<Model>>` containing `steps + 1` elements.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if either the model or the integrator fails.
    fn step_many(
        &self,
        initial_input: <Self::Model as Component>::Input,
        steps: usize,
        dt: Duration,
    ) -> Result<Vec<State<Self::Model>>, StepError<Self>>
    where
        <Self::Model as Component>::Input: Clone,
        <Self::Model as Component>::Output: Clone,
    {
        self.step_iter(initial_input, dt).take(steps + 1).collect()
    }
}

/// Represents the full state of a simulation at a specific point in time.
///
/// A [`State`] holds a pair of:
/// - `input`: The independent (user-controlled or time-evolving) variables.
/// - `output`: The dependent (model-computed) variables.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct State<C: Component> {
    pub input: C::Input,
    pub output: C::Output,
}

/// Represents an error that may occur during a simulation step.
///
/// A [`StepError`] wraps failures from either the model or the integrator.
#[derive(Debug, Error)]
pub enum StepError<S: Simulation> {
    #[error("Model failed: {0}")]
    Model(<S::Model as Component>::Error),
    #[error("Integrator failed: {0}")]
    Integrator(<S::Integrator as Integrator>::Error),
}

/// An iterator that advances a simulation one step at a time.
struct StepIter<'a, S: Simulation>
where
    <S::Model as Component>::Input: Clone,
    <S::Model as Component>::Output: Clone,
{
    sim: &'a S,
    input: Option<<S::Model as Component>::Input>,
    output: Option<<S::Model as Component>::Output>,
    dt: Duration,
}

impl<S> Iterator for StepIter<'_, S>
where
    S: Simulation,
    <S::Model as Component>::Input: Clone,
    <S::Model as Component>::Output: Clone,
{
    type Item = Result<State<S::Model>, StepError<S>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_input = self.input.take()?;

        let current_output = match self.output.take() {
            Some(output) => output,
            None => match self.sim.model().call(current_input.clone()) {
                Ok(output) => output,
                Err(error) => {
                    self.input = None;
                    return Some(Err(StepError::Model(error)));
                }
            },
        };

        let current_state = State {
            input: current_input,
            output: current_output,
        };

        match self.sim.step_from_state(&current_state, self.dt) {
            Ok(State { input, output }) => {
                self.input = Some(input);
                self.output = Some(output);
                Some(Ok(current_state))
            }
            Err(error) => {
                self.input = None;
                Some(Err(error))
            }
        }
    }
}

impl<S> FusedIterator for StepIter<'_, S>
where
    S: Simulation,
    <S::Model as Component>::Input: Clone,
    <S::Model as Component>::Output: Clone,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::{assert_abs_diff_eq, assert_relative_eq};

    use crate::Integrator;

    /// A simple spring-damper model used for simulation tests.
    #[derive(Debug)]
    struct Spring {
        spring_constant: f64,
        damping_coef: f64,
    }

    /// Test input type representing simulation state variables.
    #[derive(Debug, Clone, Default, PartialEq)]
    struct Input {
        time_in_minutes: f64,
        position: f64,
        velocity: f64,
    }

    /// Test output type representing model-computed quantities.
    #[derive(Debug, Clone, PartialEq)]
    struct Output {
        acceleration: f64,
    }

    impl Component for Spring {
        type Input = Input;
        type Output = Output;
        type Error = Infallible;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            let Input {
                position, velocity, ..
            } = input;

            let acceleration = -self.spring_constant * position - self.damping_coef * velocity;

            Ok(Output { acceleration })
        }
    }

    /// A basic forward Euler integrator for testing.
    #[derive(Debug)]
    struct FwdEuler;

    struct FwdEulerInput {
        position: f64,
        velocity: f64,
        acceleration: f64,
    }

    struct FwdEulerOutput {
        position: f64,
        velocity: f64,
    }

    impl Integrator for FwdEuler {
        type Input = FwdEulerInput;
        type Output = FwdEulerOutput;
        type Error = Infallible;

        fn integrate(
            &self,
            input: Self::Input,
            dt: Duration,
        ) -> Result<(Self::Output, Duration), Self::Error> {
            let dt_secs = dt.as_secs_f64();

            let output = FwdEulerOutput {
                position: input.position + input.velocity * dt_secs,
                velocity: input.velocity + input.acceleration * dt_secs,
            };

            Ok((output, dt))
        }
    }

    /// A simulation that combines the `Spring` model with the `FwdEuler` integrator.
    #[derive(Debug)]
    struct SpringSimulation {
        model: Spring,
        integrator: FwdEuler,
    }

    impl Simulation for SpringSimulation {
        type Model = Spring;
        type Integrator = FwdEuler;

        fn model(&self) -> &Self::Model {
            &self.model
        }

        fn integrator(&self) -> &Self::Integrator {
            &self.integrator
        }

        fn prepare_integrator_input(&self, state: &State<Spring>) -> FwdEulerInput {
            FwdEulerInput {
                position: state.input.position,
                velocity: state.input.velocity,
                acceleration: state.output.acceleration,
            }
        }

        fn prepare_model_input(
            &self,
            prev_state: &State<Spring>,
            integrator_output: FwdEulerOutput,
            actual_dt: Duration,
        ) -> Input {
            let FwdEulerOutput { position, velocity } = integrator_output;

            let minutes = actual_dt.as_secs_f64() / 60.0;
            let time_in_minutes = prev_state.input.time_in_minutes + minutes;

            Input {
                time_in_minutes,
                position,
                velocity,
            }
        }
    }

    #[test]
    fn zero_force_spring_has_constant_velocity() {
        let sim = SpringSimulation {
            model: Spring {
                spring_constant: 0.0,
                damping_coef: 0.0,
            },
            integrator: FwdEuler,
        };

        let initial = Input {
            time_in_minutes: 0.0,
            position: 10.0,
            velocity: 2.0,
        };

        let steps = 3;
        let dt = Duration::from_secs(30);

        let states = sim.step_many(initial, steps, dt).unwrap();

        let input_states: Vec<_> = states.iter().map(|s| s.input.clone()).collect();

        assert_eq!(
            input_states,
            vec![
                Input {
                    time_in_minutes: 0.0,
                    position: 10.0,
                    velocity: 2.0
                },
                Input {
                    time_in_minutes: 0.5,
                    position: 70.0,
                    velocity: 2.0
                },
                Input {
                    time_in_minutes: 1.0,
                    position: 130.0,
                    velocity: 2.0
                },
                Input {
                    time_in_minutes: 1.5,
                    position: 190.0,
                    velocity: 2.0
                },
            ]
        );

        assert!(
            states.iter().all(|s| s.output.acceleration == 0.0),
            "All accelerations should be zero"
        );
    }

    #[test]
    fn damped_spring_sim_converges_to_zero() {
        let sim = SpringSimulation {
            model: Spring {
                spring_constant: 0.5,
                damping_coef: 5.0,
            },
            integrator: FwdEuler,
        };

        let initial = Input {
            position: 10.0,
            ..Input::default()
        };

        let dt = Duration::from_millis(100);

        let tolerance = 1e-4;
        let max_steps = 5000;

        let is_at_rest = |state: &State<Spring>| {
            state.input.position.abs() < tolerance
                && state.input.velocity.abs() < tolerance
                && state.output.acceleration.abs() < tolerance
        };

        // Use the step iterator to find the first state close enough to zero.
        let final_state = sim
            .step_iter(initial, dt)
            .take(max_steps)
            .find_map(|res| match res {
                Ok(state) if is_at_rest(&state) => Some(state),
                Ok(_) => None,
                Err(error) => panic!("Simulation error: {error:?}"),
            })
            .expect("Simulation did not reach a resting state within {max_steps} steps");

        let State {
            input: final_input,
            output: final_output,
        } = final_state;

        assert_abs_diff_eq!(final_input.position, 0.0, epsilon = tolerance);
        assert_abs_diff_eq!(final_input.velocity, 0.0, epsilon = tolerance);
        assert_abs_diff_eq!(final_output.acceleration, 0.0, epsilon = tolerance);

        assert_relative_eq!(final_input.time_in_minutes, 1.875, epsilon = tolerance);
    }
}
