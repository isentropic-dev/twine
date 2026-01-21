use std::{iter::FusedIterator, time::Duration};

use crate::model::Model;

/// Trait for defining a transient simulation in Twine.
///
/// A `Simulation` advances a [`Model`] forward in time by computing its next
/// input with [`advance_time`] and then calling the model to produce a new
/// [`State`] representing the system at the corresponding future moment.
///
/// # Stepping Methods
///
/// After implementing [`advance_time`], the following methods are available for
/// advancing the simulation:
///
/// - [`Simulation::step`]: Takes a single step from an initial input.
/// - [`Simulation::step_from_state`]: Takes a single step from a known state.
/// - [`Simulation::step_many`]: Takes multiple steps and collects all resulting states.
/// - [`Simulation::into_step_iter`]: Consumes the simulation and returns an iterator over its steps.
pub trait Simulation<M: Model>: Sized {
    /// The error type returned if a simulation step fails.
    ///
    /// This type must implement [`From<M::Error>`] so errors produced by the model
    /// (via [`Model::call`]) can be automatically converted using the `?` operator.
    /// This requirement allows simulations to propagate model errors cleanly
    /// when calling the model during a step or within [`advance_time`].
    ///
    /// Implementations may:
    /// - Reuse the model's error type directly (`type StepError = M::Error`).
    /// - Wrap it in a custom enum with additional error variants.
    /// - Use boxed dynamic errors for maximum flexibility.
    type StepError: std::error::Error + Send + Sync + 'static + From<M::Error>;

    /// Provides a reference to the model being simulated.
    fn model(&self) -> &M;

    /// Computes the next input for the model, advancing the simulation in time.
    ///
    /// Given the current [`State`] and a proposed time step `dt`, this method
    /// generates the next [`Model::Input`] to drive the simulation forward.
    ///
    /// This method is the primary customization point for incrementing time,
    /// integrating state variables, enforcing constraints, applying control
    /// logic, or incorporating external events.
    /// It takes `&mut self` to support stateful integration algorithms such as
    /// adaptive time stepping, multistep methods, or PID controllers that need
    /// to record history.
    ///
    /// Implementations may interpret or adapt the proposed time step `dt` as
    /// needed (e.g., for adaptive time stepping), and are free to update any
    /// fields of the input required to continue the simulation.
    ///
    /// # Parameters
    ///
    /// - `state`: The current simulation state.
    /// - `dt`: The proposed time step.
    ///
    /// # Returns
    ///
    /// The next input, computed from the current [`State`] and proposed `dt`.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if computing the next input fails.
    fn advance_time(&mut self, state: &State<M>, dt: Duration)
    -> Result<M::Input, Self::StepError>;

    /// Advances the simulation by one step, starting from an initial input.
    ///
    /// This method first calls the model with the given input to compute
    /// the initial output, forming a complete [`State`].
    /// It then delegates to [`step_from_state`] to compute the next state.
    /// As a result, the model is called twice: once to initialize the
    /// state, and once after advancing.
    ///
    /// # Parameters
    ///
    /// - `input`: The model input at the start of the step.
    /// - `dt`: The proposed time step.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if computing the next input or calling the model fails.
    fn step(&mut self, input: M::Input, dt: Duration) -> Result<State<M>, Self::StepError> {
        let output = self.model().call(&input)?;
        let state = State::new(input, output);

        self.step_from_state(&state, dt)
    }

    /// Advances the simulation by one step from a known [`State`].
    ///
    /// This method computes the next input using [`advance_time`],
    /// then calls the model to produce the resulting [`State`].
    ///
    /// # Parameters
    ///
    /// - `state`: The current simulation state.
    /// - `dt`: The proposed time step.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if computing the next input or calling the model fails.
    fn step_from_state(
        &mut self,
        state: &State<M>,
        dt: Duration,
    ) -> Result<State<M>, Self::StepError> {
        let input = self.advance_time(state, dt)?;
        let output = self.model().call(&input)?;

        Ok(State::new(input, output))
    }

    /// Runs the simulation for a fixed number of steps and collects the results.
    ///
    /// Starting from the given input, this method advances the simulation by
    /// `steps` iterations using the proposed time step `dt`.
    ///
    /// # Parameters
    ///
    /// - `initial_input`: The model input at the start of the simulation.
    /// - `steps`: The number of steps to run.
    /// - `dt`: The proposed time step for each iteration.
    ///
    /// # Returns
    ///
    /// A `Vec` of length `steps + 1` containing each [`State`] computed during
    /// the run, including the initial one.
    ///
    /// # Errors
    ///
    /// Returns a [`StepError`] if any step fails.
    /// No further steps are taken after an error.
    fn step_many(
        &mut self,
        initial_input: M::Input,
        steps: usize,
        dt: Duration,
    ) -> Result<Vec<State<M>>, Self::StepError> {
        let mut results = Vec::with_capacity(steps + 1);

        let output = self.model().call(&initial_input)?;
        results.push(State::new(initial_input, output));

        for _ in 0..steps {
            let last = results.last().expect("results not empty");
            let next = self.step_from_state(last, dt)?;
            results.push(next);
        }

        Ok(results)
    }

    /// Consumes the simulation and creates an iterator that advances it repeatedly.
    ///
    /// The iterator calls the simulation's stepping logic with a constant `dt`,
    /// yielding each resulting [`State`] in sequence.
    /// If a step fails, the error is returned and iteration stops.
    ///
    /// This method supports lazy or streaming evaluation and integrates cleanly
    /// with iterator adapters such as `.take(n)`, `.map(...)`, or `.find(...)`.
    /// It is memory-efficient and performs no intermediate allocations.
    ///
    /// # Parameters
    ///
    /// - `initial_input`: The model input at the start of the simulation.
    /// - `dt`: The proposed time step for each iteration.
    ///
    /// # Returns
    ///
    /// An iterator over `Result<State<M>, StepError>` steps.
    fn into_step_iter(
        self,
        initial_input: M::Input,
        dt: Duration,
    ) -> impl Iterator<Item = Result<State<M>, Self::StepError>>
    where
        M::Input: Clone,
        M::Output: Clone,
    {
        StepIter {
            dt,
            known: Some(Known::Input(initial_input)),
            sim: self,
        }
    }
}

/// Represents a snapshot of the simulation at a specific point in time.
///
/// A [`State`] pairs:
/// - `input`: The independent variables, typically user-controlled or time-evolving.
/// - `output`: The dependent variables, computed by the model.
///
/// Together, these describe the full state of the system at a given instant.
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub struct State<M: Model> {
    pub input: M::Input,
    pub output: M::Output,
}

impl<M: Model> State<M> {
    /// Creates a [`State`] from the provided input and output.
    pub fn new(input: M::Input, output: M::Output) -> Self {
        Self { input, output }
    }
}

impl<M: Model> Clone for State<M>
where
    M::Input: Clone,
    M::Output: Clone,
{
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            output: self.output.clone(),
        }
    }
}

impl<M: Model> Copy for State<M>
where
    M::Input: Copy,
    M::Output: Copy,
{
}

/// An iterator that repeatedly steps the simulation using a proposed time step.
///
/// Starting from an initial input, this iterator repeatedly steps the
/// simulation using `dt`, yielding each resulting [`State`] as a `Result`.
///
/// If any step fails, the error is yielded and iteration stops.
struct StepIter<M: Model, S: Simulation<M>> {
    dt: Duration,
    known: Option<Known<M>>,
    sim: S,
}

/// Internal state held by the [`StepIter`] iterator.
enum Known<M: Model> {
    /// The simulation has only been initialized with an input.
    Input(M::Input),
    /// The full simulation state is available.
    State(State<M>),
}

impl<M, S> Iterator for StepIter<M, S>
where
    M: Model,
    S: Simulation<M>,
    M::Input: Clone,
    M::Output: Clone,
{
    type Item = Result<State<M>, S::StepError>;

    /// Advances the simulation by one step.
    ///
    /// - If starting from an input, calls the model to produce the first state.
    /// - If continuing from a full state, steps the simulation forward.
    /// - On success, yields a new [`State`].
    /// - On error, yields a [`StepError`] and ends the iteration.
    fn next(&mut self) -> Option<Self::Item> {
        let known = self.known.take()?;

        match known {
            // A full state exists - step forward from it.
            Known::State(state) => match self.sim.step_from_state(&state, self.dt) {
                Ok(state) => {
                    self.known = Some(Known::State(State::new(
                        state.input.clone(),
                        state.output.clone(),
                    )));
                    Some(Ok(state))
                }
                Err(error) => {
                    self.known = None;
                    Some(Err(error))
                }
            },

            // Only the input is known - call the model and yield the first state.
            Known::Input(input) => match self.sim.model().call(&input) {
                Ok(output) => {
                    self.known = Some(Known::State(State::new(input.clone(), output.clone())));
                    let state = State::new(input, output);
                    Some(Ok(state))
                }
                Err(error) => {
                    self.known = None;
                    Some(Err(error.into()))
                }
            },
        }
    }
}

/// Marks that iteration always ends after the first `None`.
impl<M, S> FusedIterator for StepIter<M, S>
where
    M: Model,
    S: Simulation<M>,
    M::Input: Clone,
    M::Output: Clone,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::{assert_abs_diff_eq, assert_relative_eq};
    use thiserror::Error;

    /// A simple spring-damper model used for simulation tests.
    #[derive(Debug)]
    struct Spring {
        spring_constant: f64,
        damping_coef: f64,
    }

    #[derive(Debug, Clone, Default, PartialEq)]
    struct Input {
        time_in_minutes: f64,
        position: f64,
        velocity: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Output {
        acceleration: f64,
    }

    impl Model for Spring {
        type Input = Input;
        type Output = Output;
        type Error = Infallible;

        fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
            let Input {
                position, velocity, ..
            } = input;

            let acceleration = -self.spring_constant * position - self.damping_coef * velocity;

            Ok(Output { acceleration })
        }
    }

    #[derive(Debug)]
    struct SpringSimulation {
        model: Spring,
    }

    impl Simulation<Spring> for SpringSimulation {
        type StepError = Infallible;

        fn model(&self) -> &Spring {
            &self.model
        }

        fn advance_time(
            &mut self,
            state: &State<Spring>,
            dt: Duration,
        ) -> Result<Input, Self::StepError> {
            let seconds = dt.as_secs_f64();
            let time_in_minutes = state.input.time_in_minutes + seconds / 60.0;

            let position = state.input.position + state.input.velocity * seconds;
            let velocity = state.input.velocity + state.output.acceleration * seconds;

            Ok(Input {
                time_in_minutes,
                position,
                velocity,
            })
        }
    }

    #[test]
    fn zero_force_spring_has_constant_velocity() {
        let mut sim = SpringSimulation {
            model: Spring {
                spring_constant: 0.0,
                damping_coef: 0.0,
            },
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
            .into_step_iter(initial, dt)
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

    /// A model that fails if the input exceeds a specified maximum.
    #[derive(Debug)]
    struct CheckInput {
        max_value: usize,
    }

    impl Model for CheckInput {
        type Input = usize;
        type Output = ();
        type Error = CheckInputError;

        fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
            if *input <= self.max_value {
                Ok(())
            } else {
                Err(CheckInputError(*input, self.max_value))
            }
        }
    }

    #[derive(Debug, Error)]
    #[error("{0} is bigger than max value of {1}")]
    struct CheckInputError(usize, usize);

    /// A test simulation using [`CheckInput`].
    ///
    /// Each step increments the input by 1.
    /// Yields an error when the input exceeds the maximum threshold `N`.
    #[derive(Debug)]
    struct CheckInputSim<const N: usize>;

    impl<const N: usize> Simulation<CheckInput> for CheckInputSim<N> {
        type StepError = CheckInputError;

        fn model(&self) -> &CheckInput {
            &CheckInput { max_value: N }
        }

        fn advance_time(
            &mut self,
            state: &State<CheckInput>,
            _dt: Duration,
        ) -> Result<usize, Self::StepError> {
            Ok(state.input + 1)
        }
    }

    #[test]
    fn step_iter_yields_error_correctly() {
        let mut iter = CheckInputSim::<3>.into_step_iter(0, Duration::from_secs(1));

        let state = iter
            .next()
            .expect("Initial call yields a result")
            .expect("Initial call is a success");
        assert_eq!(state.input, 0);

        let state = iter
            .next()
            .expect("First step yields a result")
            .expect("First step is a success");
        assert_eq!(state.input, 1);

        let state = iter
            .next()
            .expect("Second step yields a result")
            .expect("Second step is a success");
        assert_eq!(state.input, 2);

        let state = iter
            .next()
            .expect("Third step yields a result")
            .expect("Third step is a success");
        assert_eq!(state.input, 3);

        let error = iter
            .next()
            .expect("Fourth step yields a result")
            .expect_err("Fourth step is an error");
        assert_eq!(format!("{error}"), "4 is bigger than max value of 3");

        assert!(iter.next().is_none());
    }
}
