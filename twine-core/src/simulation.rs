use std::{iter::FusedIterator, time::Duration};

/// Trait for defining a system model in Twine.
///
/// Models must be deterministic, always producing the same result for a given input.
pub trait Model {
    type Input;
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Calls the model with the given input and returns a result.
    ///
    /// # Errors
    ///
    /// Each model defines its own `Error` type, allowing it to determine what
    /// constitutes a failure within its domain.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Trait for simulating the time evolution of a [`Model`].
///
/// A `Simulation` advances a model forward in time by computing its next
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
/// - [`Simulation::step_iter`]: Returns an iterator over simulation steps.
pub trait Simulation: Sized {
    /// The [`Model`] being simulated.
    type Model: Model;

    /// The error type returned if a simulation step fails.
    type StepError: std::error::Error + From<<Self::Model as Model>::Error> + Send + Sync + 'static;

    /// Provides a reference to the model being simulated.
    fn model(&self) -> &Self::Model;

    /// Computes the next input for the model, advancing the simulation in time.
    ///
    /// Given the current [`State`] and a proposed time step `dt`, this method
    /// generates the next [`Model::Input`] to drive the simulation forward.
    ///
    /// This method is the primary customization point for incrementing time,
    /// integrating state variables, enforcing constraints, applying control
    /// logic, or incorporating external events.
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
    fn advance_time(
        &self,
        state: &State<Self::Model>,
        dt: Duration,
    ) -> Result<<Self::Model as Model>::Input, Self::StepError>;

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
    /// Returns a [`StepError`] if computing the next input or calling the
    /// model fails.
    fn step(
        &self,
        input: <Self::Model as Model>::Input,
        dt: Duration,
    ) -> Result<State<Self::Model>, Self::StepError>
    where
        <Self::Model as Model>::Input: Clone,
    {
        let output = self.model().call(input.clone())?;
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
    /// Returns a [`StepError`] if computing the next input or calling the
    /// model fails.
    fn step_from_state(
        &self,
        state: &State<Self::Model>,
        dt: Duration,
    ) -> Result<State<Self::Model>, Self::StepError>
    where
        <Self::Model as Model>::Input: Clone,
    {
        let input = self.advance_time(state, dt)?;
        let output = self.model().call(input.clone())?;

        Ok(State::new(input, output))
    }

    /// Creates an iterator that advances the simulation repeatedly.
    ///
    /// The iterator calls [`step`] with a constant `dt`, yielding each
    /// resulting [`State`] in sequence.
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
    /// An iterator over `Result<State<Model>, StepError>`.
    fn step_iter(
        &self,
        initial_input: <Self::Model as Model>::Input,
        dt: Duration,
    ) -> impl Iterator<Item = Result<State<Self::Model>, Self::StepError>>
    where
        <Self::Model as Model>::Input: Clone,
        <Self::Model as Model>::Output: Clone,
    {
        StepIter {
            dt,
            known: Some(Known::Input(initial_input)),
            sim: self,
        }
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
        &self,
        initial_input: <Self::Model as Model>::Input,
        steps: usize,
        dt: Duration,
    ) -> Result<Vec<State<Self::Model>>, Self::StepError>
    where
        <Self::Model as Model>::Input: Clone,
        <Self::Model as Model>::Output: Clone,
    {
        self.step_iter(initial_input, dt).take(steps + 1).collect()
    }
}

/// Represents a snapshot of the simulation at a specific point in time.
///
/// A [`State`] pairs:
/// - `input`: The independent variables, typically user-controlled or time-evolving.
/// - `output`: The dependent variables, computed by the model.
///
/// Together, these describe the full state of the system at a given instant.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct State<C: Model> {
    pub input: C::Input,
    pub output: C::Output,
}

impl<C: Model> State<C> {
    /// Creates a [`State`] from the provided input and output.
    pub fn new(input: C::Input, output: C::Output) -> Self {
        Self { input, output }
    }
}

/// An iterator that repeatedly steps the simulation using a proposed time step.
///
/// Starting from an initial input, this iterator repeatedly steps the
/// simulation using `dt`, yielding each resulting [`State`] as a `Result`.
///
/// If any step fails, the error is yielded and iteration stops.
struct StepIter<'a, S: Simulation> {
    dt: Duration,
    known: Option<Known<S>>,
    sim: &'a S,
}

/// Internal state held by the [`StepIter`] iterator.
enum Known<S: Simulation> {
    /// The simulation has only been initialized with an input.
    Input(<S::Model as Model>::Input),
    /// The full simulation state is available.
    State(State<S::Model>),
}

impl<S> Iterator for StepIter<'_, S>
where
    S: Simulation,
    <S::Model as Model>::Input: Clone,
    <S::Model as Model>::Output: Clone,
{
    type Item = Result<State<S::Model>, S::StepError>;

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
            Known::Input(input) => match self.sim.model().call(input.clone()) {
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
impl<S> FusedIterator for StepIter<'_, S>
where
    S: Simulation,
    <S::Model as Model>::Input: Clone,
    <S::Model as Model>::Output: Clone,
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

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
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

    impl Simulation for SpringSimulation {
        type Model = Spring;
        type StepError = Infallible;

        fn model(&self) -> &Self::Model {
            &self.model
        }

        fn advance_time(
            &self,
            state: &State<Self::Model>,
            dt: Duration,
        ) -> Result<<Self::Model as Model>::Input, Self::StepError> {
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
        let sim = SpringSimulation {
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

    /// A model that fails if the input exceeds a specified maximum.
    #[derive(Debug)]
    struct CheckInput {
        max_value: usize,
    }

    impl Model for CheckInput {
        type Input = usize;
        type Output = ();
        type Error = CheckInputError;

        fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
            if input <= self.max_value {
                Ok(())
            } else {
                Err(CheckInputError(input, self.max_value))
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

    impl<const N: usize> Simulation for CheckInputSim<N> {
        type Model = CheckInput;
        type StepError = CheckInputError;

        fn model(&self) -> &Self::Model {
            &CheckInput { max_value: N }
        }

        fn advance_time(
            &self,
            state: &State<Self::Model>,
            _dt: Duration,
        ) -> Result<<Self::Model as Model>::Input, Self::StepError> {
            Ok(state.input + 1)
        }
    }

    #[test]
    fn step_iter_yields_error_correctly() {
        let mut iter = CheckInputSim::<3>.step_iter(0, Duration::from_secs(1));

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
