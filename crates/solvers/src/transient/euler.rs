//! Forward Euler solver for ODE problems.
//!
//! This module provides a minimal forward Euler integrator for transient simulations.
//! It steps a model forward in time using explicit Euler:
//!
//! ```text
//! state_{n+1} = state_n + derivative_n * dt
//! ```
//!
//! # Example
//!
//! ```ignore
//! use twine_solvers::transient::euler;
//!
//! let solution = euler::solve_unobserved(&model, &problem, initial_input, dt, steps)?;
//!
//! for snapshot in &solution.history {
//!     println!("t={}: {:?}", snapshot.input, snapshot.output);
//! }
//! ```

mod action;
mod error;
mod event;
mod solution;

pub use action::Action;
pub use error::Error;
pub use event::Event;
pub use solution::{Solution, Status};

use twine_core::{Model, Observer, OdeProblem, Snapshot, StepIntegrable};

/// Integrates an ODE problem using forward Euler.
///
/// # Algorithm
///
/// 1. Call the model with the initial input to get the initial snapshot.
/// 2. For each step:
///    - Extract the state from the current input.
///    - Compute the derivative from the current input and output.
///    - Step the state forward: `state + derivative * dt`.
///    - Build the next input from the stepped state.
///    - Finalize the step (for discrete controls, constraints, etc.).
///    - Call the model to get the next output.
///    - Emit a `Event` to the observer.
///    - If the observer returns `StopEarly`, terminate.
/// 3. Return the solution with the full history.
///
/// # Observer
///
/// The observer receives a [`Event`] after each integration step and may
/// return [`Action::StopEarly`] to terminate the simulation early.
///
/// # Errors
///
/// Returns an error if the model or problem returns an error at any point.
pub fn solve<M, P, Obs>(
    model: &M,
    problem: &P,
    initial: M::Input,
    dt: P::Delta,
    steps: usize,
    mut observer: Obs,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: OdeProblem<Input = M::Input, Output = M::Output>,
    P::Delta: Clone,
    Obs: Observer<Event<M::Input, M::Output>, Action>,
{
    // Evaluate initial state.
    let initial_output = model.call(&initial).map_err(Error::model)?;
    let initial_snapshot = Snapshot::new(initial, initial_output);

    let mut history = Vec::with_capacity(steps + 1);
    history.push(initial_snapshot.clone());

    // Emit initial event.
    let event = Event {
        step: 0,
        snapshot: initial_snapshot.clone(),
    };
    if let Some(Action::StopEarly) = observer.observe(&event) {
        return Ok(Solution {
            status: Status::StoppedByObserver,
            history,
            steps: 0,
        });
    }

    let mut current = initial_snapshot;

    for step in 1..=steps {
        // Extract state and compute derivative.
        let state = problem.state(&current.input).map_err(Error::problem)?;
        let derivative = problem
            .derivative(&current.input, &current.output)
            .map_err(Error::problem)?;

        // Step state forward.
        let next_state = state.step(derivative, dt.clone());

        // Build and finalize next input.
        let next_input = problem
            .build_input(&current.input, &next_state, &dt)
            .map_err(Error::problem)?;
        let next_input = problem
            .finalize_step(next_input, &current.input, &current.output, &dt)
            .map_err(Error::problem)?;

        // Evaluate model at next state.
        let next_output = model.call(&next_input).map_err(Error::model)?;
        let next_snapshot = Snapshot::new(next_input, next_output);

        history.push(next_snapshot.clone());

        // Emit event to observer.
        let event = Event {
            step,
            snapshot: next_snapshot.clone(),
        };

        if let Some(Action::StopEarly) = observer.observe(&event) {
            return Ok(Solution {
                status: Status::StoppedByObserver,
                history,
                steps: step,
            });
        }

        current = next_snapshot;
    }

    Ok(Solution {
        status: Status::Complete,
        history,
        steps,
    })
}

/// Integrates an ODE problem using forward Euler without observation.
///
/// This is a convenience wrapper around [`solve`] that discards events.
///
/// # Errors
///
/// Returns an error if the model or problem returns an error at any point.
pub fn solve_unobserved<M, P>(
    model: &M,
    problem: &P,
    initial: M::Input,
    dt: P::Delta,
    steps: usize,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: OdeProblem<Input = M::Input, Output = M::Output>,
    P::Delta: Clone,
{
    solve(model, problem, initial, dt, steps, ())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::assert_relative_eq;
    use twine_core::DerivativeOf;

    // --- Test fixtures ---

    /// State: position
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position(f64);

    /// Derivative: velocity
    #[derive(Debug, Clone, Copy)]
    struct Velocity(f64);

    impl StepIntegrable<f64> for Position {
        type Derivative = Velocity;

        fn step(&self, derivative: Velocity, dt: f64) -> Self {
            Position(self.0 + derivative.0 * dt)
        }
    }

    /// Model input: current position and time.
    #[derive(Debug, Clone)]
    struct Input {
        position: Position,
        time: f64,
    }

    /// Model output: velocity at the current state.
    #[derive(Debug, Clone)]
    struct Output {
        velocity: Velocity,
    }

    /// Model with constant velocity.
    struct ConstantVelocityModel {
        velocity: f64,
    }

    impl Model for ConstantVelocityModel {
        type Input = Input;
        type Output = Output;
        type Error = Infallible;

        fn call(&self, _input: &Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(Output {
                velocity: Velocity(self.velocity),
            })
        }
    }

    /// Problem that extracts position and velocity.
    struct MotionProblem;

    impl OdeProblem for MotionProblem {
        type Input = Input;
        type Output = Output;
        type Delta = f64;
        type State = Position;
        type Error = Infallible;

        fn state(&self, input: &Self::Input) -> Result<Self::State, Self::Error> {
            Ok(input.position)
        }

        fn derivative(
            &self,
            _input: &Self::Input,
            output: &Self::Output,
        ) -> Result<DerivativeOf<Self::State, Self::Delta>, Self::Error> {
            Ok(output.velocity)
        }

        fn build_input(
            &self,
            base: &Self::Input,
            state: &Self::State,
            delta: &Self::Delta,
        ) -> Result<Self::Input, Self::Error> {
            Ok(Input {
                position: *state,
                time: base.time + delta,
            })
        }
    }

    // --- Tests ---

    #[test]
    fn constant_velocity_motion() {
        let model = ConstantVelocityModel { velocity: 2.0 };
        let problem = MotionProblem;
        let initial = Input {
            position: Position(0.0),
            time: 0.0,
        };

        let solution = solve_unobserved(&model, &problem, initial, 0.1, 10).expect("should solve");

        assert_eq!(solution.status, Status::Complete);
        assert_eq!(solution.steps, 10);
        assert_eq!(solution.history.len(), 11); // initial + 10 steps

        // After 10 steps at v=2, dt=0.1: position = 0 + 2*0.1*10 = 2.0
        let final_snapshot = solution.history.last().unwrap();
        assert_relative_eq!(final_snapshot.input.position.0, 2.0);
        assert_relative_eq!(final_snapshot.input.time, 1.0);
    }

    #[test]
    fn observer_can_stop_early() {
        let model = ConstantVelocityModel { velocity: 1.0 };
        let problem = MotionProblem;
        let initial = Input {
            position: Position(0.0),
            time: 0.0,
        };

        let observer = |event: &Event<Input, Output>| {
            if event.step >= 5 {
                Some(Action::StopEarly)
            } else {
                None
            }
        };

        let solution =
            solve(&model, &problem, initial, 0.1, 100, observer).expect("should stop early");

        assert_eq!(solution.status, Status::StoppedByObserver);
        assert_eq!(solution.steps, 5);
        assert_eq!(solution.history.len(), 6); // initial + 5 steps
    }

    #[test]
    fn zero_steps_returns_initial() {
        let model = ConstantVelocityModel { velocity: 1.0 };
        let problem = MotionProblem;
        let initial = Input {
            position: Position(5.0),
            time: 0.0,
        };

        let solution =
            solve_unobserved(&model, &problem, initial, 0.1, 0).expect("should return initial");

        assert_eq!(solution.status, Status::Complete);
        assert_eq!(solution.steps, 0);
        assert_eq!(solution.history.len(), 1);
        assert_relative_eq!(solution.history[0].input.position.0, 5.0);
    }

    #[test]
    fn step_numbers_start_at_zero() {
        let model = ConstantVelocityModel { velocity: 1.0 };
        let problem = MotionProblem;
        let initial = Input {
            position: Position(0.0),
            time: 0.0,
        };

        let mut step_values = Vec::new();
        solve(
            &model,
            &problem,
            initial,
            0.25,
            4,
            |event: &Event<Input, Output>| {
                step_values.push(event.step);
                None
            },
        )
        .expect("should solve");

        assert_eq!(step_values, vec![0, 1, 2, 3, 4]);
    }
}
