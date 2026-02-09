use std::convert::Infallible;

use approx::assert_relative_eq;
use thiserror::Error;

use twine_core::{Model, OptimizationProblem};

use super::{
    Action, Config, Error, Event, Status, maximize_unobserved, minimize, minimize_unobserved,
};

/// A simple polynomial: f(x) = x³ - 4x.
struct Polynomial;

impl Model for Polynomial {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        Ok(x.powi(3) - 4.0 * x)
    }
}

/// Objective: just use the model output as the objective.
struct ObjectiveOutput;

impl OptimizationProblem<1> for ObjectiveOutput {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<Self::Input, Self::Error> {
        Ok(x[0])
    }

    fn objective(&self, _input: &f64, output: &f64) -> Result<f64, Self::Error> {
        Ok(*output)
    }
}

#[test]
fn minimizes_polynomial() {
    let model = Polynomial;
    let problem = ObjectiveOutput;

    // Local minimum at x = 2/sqrt(3) ≈ 1.1547.
    let expected_x = 2.0 / 3.0_f64.sqrt();

    let solution = minimize_unobserved(&model, &problem, [-2.0, 2.0], &Config::default())
        .expect("should converge");

    assert_eq!(solution.status, Status::Converged);
    assert_relative_eq!(solution.x, expected_x, epsilon = 1e-8);
}

#[test]
fn maximizes_polynomial() {
    let model = Polynomial;
    let problem = ObjectiveOutput;

    // Local maximum at x = -2/sqrt(3) ≈ -1.1547.
    let expected_x = -2.0 / 3.0_f64.sqrt();

    let solution = maximize_unobserved(&model, &problem, [-2.0, 2.0], &Config::default())
        .expect("should converge");

    assert_eq!(solution.status, Status::Converged);
    assert_relative_eq!(solution.x, expected_x, epsilon = 1e-8);
}

/// Identity model: f(x) = x.
struct Identity;

impl Model for Identity {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        Ok(*x)
    }
}

#[test]
fn assume_worse_discards_from_best() {
    // For f(x) = x on [0, 10], minimum is at x=0.
    //
    // Init: left interior (~3.82) is evaluated without observer,
    //       right interior (~6.18) goes through observer (event 1).
    //       Best after init: left (~3.82).
    //
    // Iter 1: left_score < right_score, so bracket shrinks right.
    //         New left interior (~2.36) is evaluated (event 2).
    //         This point has better objective (2.36 < 3.82).
    //
    // If we mark iteration 1's point as AssumeWorse, it shouldn't become best.

    let model = Identity;
    let problem = ObjectiveOutput;

    let mut event_count = 0;
    let observer = |_event: &Event<'_, _, _>| {
        event_count += 1;
        if event_count == 2 {
            // Event 2 is the first loop iteration - a point with better objective.
            // Mark it AssumeWorse so it's discarded from best tracking.
            Some(Action::AssumeWorse)
        } else {
            None
        }
    };

    let config = Config::new(1, 1e-12, 1e-12).unwrap();

    let solution = minimize(&model, &problem, [0.0, 10.0], &config, observer)
        .expect("should complete");

    assert_eq!(solution.status, Status::MaxIters);

    // The iteration 1 point (~2.36) has lower objective but was marked AssumeWorse.
    // Best should remain the init left point (~3.82).
    assert_relative_eq!(solution.x, 3.819_660_1, epsilon = 1e-5);
}

#[test]
fn assume_worse_steers_search() {
    // For f(x) = x on [0, 10], true minimum is at x=0.
    //
    // Init: left (~3.82) evaluated without observer, right (~6.18) via observer.
    //
    // If we mark high-x points (x > 5) as AssumeWorse:
    // - Init right (~6.18) gets score = infinity
    // - Algorithm thinks minimum is in the left region
    // - Search converges toward x=0 (correct behavior)
    //
    // If we mark low-x points (x < 5) as AssumeWorse:
    // - Init right (~6.18) is NOT marked (score = 6.18)
    // - But init left (~3.82) has score 3.82 < 6.18
    // - First iteration evaluates new left interior (~2.36), marked AssumeWorse
    // - With score = infinity, algorithm thinks minimum is on the right
    // - Search is steered RIGHT despite true minimum being at x=0
    //
    // Verify: marking low-x points as AssumeWorse steers search toward high x.

    let model = Identity;
    let problem = ObjectiveOutput;

    // Mark any point with x < 5 as AssumeWorse (except init left, which bypasses observer).
    let observer = |event: &Event<'_, _, _>| {
        if event.x() < 5.0 {
            Some(Action::AssumeWorse)
        } else {
            None
        }
    };

    let config = Config::new(20, 1e-12, 1e-12).unwrap();

    let solution = minimize(&model, &problem, [0.0, 10.0], &config, observer)
        .expect("should complete");

    // Init left (~3.82) is the only low-x point not marked AssumeWorse.
    // All other low-x evaluations get score = infinity, steering search right.
    // Final bracket should be pushed toward high x, but best remains init left
    // since no unmarked point beats it.
    //
    // Key assertion: best is the init left point, not something closer to x=0
    // (which would happen without AssumeWorse steering).
    assert_relative_eq!(solution.x, 3.819_660_1, epsilon = 1e-5);
}

/// Quadratic model: f(x) = (x - 5)².
struct Quadratic;

impl Model for Quadratic {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        Ok((x - 5.0).powi(2))
    }
}

#[test]
fn observer_can_stop_early() {
    let model = Polynomial;
    let problem = ObjectiveOutput;

    let mut eval_count = 0;
    let observer = |_event: &Event<'_, _, _>| {
        eval_count += 1;
        if eval_count >= 3 {
            Some(Action::StopEarly)
        } else {
            None
        }
    };

    let solution = minimize(&model, &problem, [0.0, 3.0], &Config::default(), observer)
        .expect("should stop cleanly");

    assert_eq!(solution.status, Status::StoppedByObserver);
    // 1 event for init right, 2 events for loop iterations = 3 total, stopped on 3rd.
    assert_eq!(solution.iters, 2);
    assert_eq!(eval_count, 3);
}

#[test]
fn assume_worse_steers_away_from_true_minimum() {
    // For f(x) = (x - 5)² on [0, 10], true minimum is at x=5.
    // Without observer, golden section would converge to x ≈ 5.
    //
    // If we mark x > 6 as AssumeWorse, search is steered left of the minimum.
    // (We use > 6 instead of > 5 because init right is at ~6.18.)

    let model = Quadratic;
    let problem = ObjectiveOutput;

    // Mark high-x points as AssumeWorse to steer search left.
    let observer = |event: &Event<'_, _, _>| {
        if event.x() > 6.0 {
            Some(Action::AssumeWorse)
        } else {
            None
        }
    };

    let config = Config::new(30, 1e-12, 1e-12).unwrap();

    let solution = minimize(&model, &problem, [0.0, 10.0], &config, observer)
        .expect("should complete");

    // Search was steered away from x > 6.
    // Without steering, solution.x would be ~5. With steering, it should be < 5.
    assert!(
        solution.x < 5.0,
        "search should be steered left of minimum, got {}",
        solution.x
    );
}

// --- Phase 5: Loop failure handling ---

/// Model that fails when x exceeds a threshold.
struct ThresholdModel {
    threshold: f64,
}

#[derive(Debug, Clone, Error)]
#[error("model failed at x={x} (threshold={threshold})")]
struct ThresholdError {
    x: f64,
    threshold: f64,
}

impl Model for ThresholdModel {
    type Input = f64;
    type Output = f64;
    type Error = ThresholdError;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        if *x > self.threshold {
            Err(ThresholdError {
                x: *x,
                threshold: self.threshold,
            })
        } else {
            // Simple parabola with minimum at x=2.
            Ok((x - 2.0).powi(2))
        }
    }
}

#[test]
fn loop_failure_stops_early() {
    // Model fails for x > 5. Bracket [0, 10] has interior points at ~3.82 and ~6.18.
    // Init left (~3.82) succeeds (no observer). Init right (~6.18) fails.
    // Observer receives ModelFailed and returns StopEarly.

    let model = ThresholdModel { threshold: 5.0 };
    let problem = ObjectiveOutput;

    let observer = |event: &Event<'_, _, _>| {
        if matches!(event, Event::ModelFailed { .. }) {
            Some(Action::StopEarly)
        } else {
            None
        }
    };

    let solution = minimize(&model, &problem, [0.0, 10.0], &Config::default(), observer)
        .expect("should stop cleanly");

    assert_eq!(solution.status, Status::StoppedByObserver);
    // Stopped during init (before any loop iterations).
    assert_eq!(solution.iters, 0);
}

#[test]
fn loop_failure_recovers_with_assume_worse() {
    // Model fails for x > 5. True minimum is at x=2.
    // Bracket [0, 10] has interior points at ~3.82 and ~6.18.
    //
    // Init right (~6.18) fails. Observer returns AssumeWorse.
    // This steers search toward the left (lower x), which is correct.
    // All subsequent failing points are also marked AssumeWorse.
    //
    // Search should converge to x ≈ 2 despite some evaluations failing.

    let model = ThresholdModel { threshold: 5.0 };
    let problem = ObjectiveOutput;

    let observer = |event: &Event<'_, _, _>| {
        if matches!(event, Event::ModelFailed { .. }) {
            Some(Action::AssumeWorse)
        } else {
            None
        }
    };

    let solution = minimize(&model, &problem, [0.0, 10.0], &Config::default(), observer)
        .expect("should recover and converge");

    assert_eq!(solution.status, Status::Converged);
    assert_relative_eq!(solution.x, 2.0, epsilon = 1e-6);
}

#[test]
fn loop_failure_without_action_errors() {
    // Model fails for x > 5. Bracket [0, 10] has init right at ~6.18 which fails.
    // Observer returns None (no action), so the solver should error.

    let model = ThresholdModel { threshold: 5.0 };
    let problem = ObjectiveOutput;

    let observer = |_event: &Event<'_, _, _>| None;

    let result = minimize(&model, &problem, [0.0, 10.0], &Config::default(), observer);

    assert!(matches!(result, Err(Error::Model(_))));
}
