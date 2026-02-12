use twine_core::{Model, Observer, OptimizationProblem};

use crate::optimization::evaluate::{EvalError, Evaluation, evaluate};

use super::{
    Action, Error, Event, Point, Solution, bracket::GoldenBracket, solution::Status, state::State,
};

pub(super) enum InitResult<I, O> {
    Continue(State<I, O>),
    StopEarly(Solution<I, O>),
}

#[allow(clippy::too_many_lines)]
/// Initialize state by evaluating both interior points.
///
/// Only the second point (or failure) triggers an observer event.
/// This is intentional: we need a valid `other` point for the event.
///
/// If both evaluations fail, we emit one failure event (with a synthetic
/// `other`) for observer awareness, then return an error. Recovery isn't
/// possible: `AssumeWorse` needs one valid point, `StopEarly` needs a snapshot.
pub(super) fn init<M, P, Obs, F>(
    model: &M,
    problem: &P,
    bracket: &GoldenBracket,
    observer: &mut Obs,
    transform: &F,
) -> Result<InitResult<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
    F: Fn(f64) -> f64,
{
    enum Outcome<I, O, ME, PE> {
        BothOk(Evaluation<I, O, 1>, Evaluation<I, O, 1>),
        OneFailed {
            ok_eval: Evaluation<I, O, 1>,
            failed_x: f64,
            err: EvalError<ME, PE>,
        },
    }

    let left = evaluate(model, problem, [bracket.inner_left]);
    let right = evaluate(model, problem, [bracket.inner_right]);

    let outcome = match (left, right) {
        (Err(left_err), Err(_)) => {
            // Emit event so observer is notified, even though recovery isn't
            // possible (AssumeWorse needs one valid point, StopEarly needs a
            // snapshot). Use synthetic `other` since both failed.
            let synthetic_other = Point::new(bracket.inner_right, f64::NAN);
            Event::emit_failure(bracket.inner_left, synthetic_other, &left_err, observer);
            return Err(left_err.into());
        }
        (Ok(l), Ok(r)) => Outcome::BothOk(l, r),
        (Ok(ok), Err(e)) => Outcome::OneFailed {
            ok_eval: ok,
            failed_x: bracket.inner_right,
            err: e,
        },
        (Err(e), Ok(ok)) => Outcome::OneFailed {
            ok_eval: ok,
            failed_x: bracket.inner_left,
            err: e,
        },
    };

    match outcome {
        Outcome::BothOk(left_eval, right_eval) => {
            let left_pt = Point::from(&left_eval);
            let right_pt = Point::from(&right_eval);
            let event = Event::Evaluated {
                point: right_pt,
                input: &right_eval.snapshot.input,
                output: &right_eval.snapshot.output,
                other: left_pt,
            };
            match observer.observe(&event) {
                Some(Action::StopEarly) => Ok(InitResult::StopEarly(Solution {
                    status: Status::StoppedByObserver,
                    x: left_pt.x,
                    objective: left_pt.objective,
                    snapshot: left_eval.snapshot,
                    iters: 0,
                })),
                Some(Action::AssumeWorse) => {
                    let worse = Point::new(right_pt.x, transform(f64::INFINITY));
                    Ok(InitResult::Continue(State::new(
                        *bracket,
                        left_pt,
                        worse,
                        left_pt,
                        left_eval.snapshot,
                    )))
                }
                None => {
                    let (best_pt, best_snap) =
                        if transform(left_pt.objective) <= transform(right_pt.objective) {
                            (left_pt, left_eval.snapshot)
                        } else {
                            (right_pt, right_eval.snapshot)
                        };
                    Ok(InitResult::Continue(State::new(
                        *bracket, left_pt, right_pt, best_pt, best_snap,
                    )))
                }
            }
        }

        Outcome::OneFailed {
            ok_eval,
            failed_x,
            err,
        } => {
            let ok_pt = Point::from(&ok_eval);
            let action = Event::emit_failure(failed_x, ok_pt, &err, observer);
            match action {
                Some(Action::StopEarly) => Ok(InitResult::StopEarly(Solution {
                    status: Status::StoppedByObserver,
                    x: ok_pt.x,
                    objective: ok_pt.objective,
                    snapshot: ok_eval.snapshot,
                    iters: 0,
                })),
                Some(Action::AssumeWorse) => {
                    let worse = Point::new(failed_x, transform(f64::INFINITY));
                    let (left_pt, right_pt) = if ok_pt.x < worse.x {
                        (ok_pt, worse)
                    } else {
                        (worse, ok_pt)
                    };
                    Ok(InitResult::Continue(State::new(
                        *bracket,
                        left_pt,
                        right_pt,
                        ok_pt,
                        ok_eval.snapshot,
                    )))
                }
                None => Err(err.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::Infallible;

    use approx::assert_relative_eq;
    use thiserror::Error;

    struct IdentityModel;

    impl Model for IdentityModel {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn call(&self, x: &f64) -> Result<f64, Self::Error> {
            Ok(*x)
        }
    }

    struct ObjectiveIsOutput;

    impl OptimizationProblem<1> for ObjectiveIsOutput {
        type Input = f64;
        type Output = f64;
        type Error = Infallible;

        fn input(&self, x: &[f64; 1]) -> Result<f64, Self::Error> {
            Ok(x[0])
        }

        fn objective(&self, _: &f64, output: &f64) -> Result<f64, Self::Error> {
            Ok(*output)
        }
    }

    fn identity_transform(x: f64) -> f64 {
        x
    }

    #[test]
    fn both_ok_selects_best() {
        let model = IdentityModel;
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let result =
            init(&model, &problem, &bracket, &mut (), &identity_transform).expect("should succeed");

        let state = match result {
            InitResult::Continue(s) => s,
            InitResult::StopEarly(_) => panic!("unexpected stop"),
        };

        // Left interior point (~3.82) has lower objective than right (~6.18)
        // So left should be best
        assert_relative_eq!(state.left().x, bracket.inner_left, epsilon = 1e-10);
        assert_relative_eq!(state.right().x, bracket.inner_right, epsilon = 1e-10);
    }

    #[test]
    fn both_ok_observer_can_stop() {
        let model = IdentityModel;
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let mut observer = |_: &Event<'_, _, _>| Some(Action::StopEarly);

        let result = init(
            &model,
            &problem,
            &bracket,
            &mut observer,
            &identity_transform,
        )
        .expect("should succeed");

        assert!(matches!(result, InitResult::StopEarly(_)));
    }

    #[test]
    fn both_ok_observer_can_assume_worse() {
        let model = IdentityModel;
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let mut observer = |_: &Event<'_, _, _>| Some(Action::AssumeWorse);

        let result = init(
            &model,
            &problem,
            &bracket,
            &mut observer,
            &identity_transform,
        )
        .expect("should succeed");

        let state = match result {
            InitResult::Continue(s) => s,
            InitResult::StopEarly(_) => panic!("unexpected stop"),
        };

        // Right was marked AssumeWorse, so right should have infinite objective
        assert!(state.right().objective.is_infinite());
        assert_relative_eq!(state.left().x, bracket.inner_left, epsilon = 1e-10);
    }

    // --- One failed tests ---

    #[derive(Debug, Error)]
    #[error("fails above {threshold}")]
    struct ThresholdError {
        threshold: f64,
    }

    struct FailsAbove {
        threshold: f64,
    }

    impl Model for FailsAbove {
        type Input = f64;
        type Output = f64;
        type Error = ThresholdError;

        fn call(&self, x: &f64) -> Result<f64, Self::Error> {
            if *x > self.threshold {
                Err(ThresholdError {
                    threshold: self.threshold,
                })
            } else {
                Ok(*x)
            }
        }
    }

    #[test]
    fn one_failed_errors_without_observer_action() {
        // Right point (~6.18) fails, left (~3.82) succeeds
        let model = FailsAbove { threshold: 5.0 };
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let result = init(&model, &problem, &bracket, &mut (), &identity_transform);

        assert!(result.is_err());
    }

    #[test]
    fn one_failed_recovers_with_assume_worse() {
        let model = FailsAbove { threshold: 5.0 };
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let mut observer = |event: &Event<'_, _, _>| {
            if matches!(event, Event::ModelFailed { .. }) {
                Some(Action::AssumeWorse)
            } else {
                None
            }
        };

        let result = init(
            &model,
            &problem,
            &bracket,
            &mut observer,
            &identity_transform,
        )
        .expect("should recover");

        let state = match result {
            InitResult::Continue(s) => s,
            InitResult::StopEarly(_) => panic!("unexpected stop"),
        };

        // Right failed and was assumed worse
        assert!(state.right().objective.is_infinite());
        assert_relative_eq!(state.left().x, bracket.inner_left, epsilon = 1e-10);
    }

    #[test]
    fn one_failed_can_stop_early() {
        let model = FailsAbove { threshold: 5.0 };
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let mut observer = |event: &Event<'_, _, _>| {
            if matches!(event, Event::ModelFailed { .. }) {
                Some(Action::StopEarly)
            } else {
                None
            }
        };

        let result = init(
            &model,
            &problem,
            &bracket,
            &mut observer,
            &identity_transform,
        )
        .expect("should succeed");

        assert!(matches!(result, InitResult::StopEarly(_)));
    }

    // --- Both failed test ---

    #[test]
    fn both_failed_returns_error() {
        let model = FailsAbove { threshold: -1.0 }; // fails everywhere
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let result = init(&model, &problem, &bracket, &mut (), &identity_transform);

        assert!(result.is_err());
    }

    #[test]
    fn both_failed_notifies_observer() {
        let model = FailsAbove { threshold: -1.0 };
        let problem = ObjectiveIsOutput;
        let bracket = GoldenBracket::new([0.0, 10.0]);

        let mut notified = false;
        let mut observer = |event: &Event<'_, _, _>| {
            if matches!(event, Event::ModelFailed { .. }) {
                notified = true;
            }
            None
        };

        let _ = init(&model, &problem, &bracket, &mut observer, &identity_transform);
        assert!(notified, "observer should be notified when both fail");
    }
}
