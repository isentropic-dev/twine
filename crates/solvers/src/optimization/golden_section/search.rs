use twine_core::{Model, Observer, OptimizationProblem, Snapshot};

use crate::optimization::evaluate::evaluate;

use super::{
    Action, Config, Error, Event, Point, Solution,
    bracket::GoldenBracket,
    init::{InitResult, init},
    solution::Status,
    state::ShrinkDirection,
};

/// Core golden section search implementation.
///
/// The `objective_transform` function is applied to objective values before
/// comparison, allowing the same algorithm to handle both minimization
/// (transform = identity) and maximization (transform = negation).
pub(super) fn search<M, P, Obs, F>(
    model: &M,
    problem: &P,
    bracket: [f64; 2],
    config: &Config,
    mut observer: Obs,
    transform: F,
) -> Result<Solution<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
    F: Fn(f64) -> f64,
{
    let bracket = GoldenBracket::new(bracket);

    let mut state = match init(model, problem, &bracket, &mut observer, &transform)? {
        InitResult::Continue(state) => state,
        InitResult::StopEarly(solution) => return Ok(solution),
    };

    for iter in 1..=config.max_iters() {
        if state.is_converged(config) {
            return Ok(state.into_solution(Status::Converged, iter - 1));
        }

        let direction = state.next_action(&transform);
        let (eval_x, other) = match direction {
            ShrinkDirection::ShrinkLeft(x) => (x, state.right()),
            ShrinkDirection::ShrinkRight(x) => (x, state.left()),
        };

        let outcome = eval_and_observe(model, problem, eval_x, other, &mut observer)?;

        let (point, snapshot) = match outcome {
            EvalOutcome::Continue { point, snapshot } => (point, Some(snapshot)),
            EvalOutcome::AssumeWorse => (Point::new(eval_x, transform(f64::INFINITY)), None),
            EvalOutcome::StopEarly => {
                return Ok(state.into_solution(Status::StoppedByObserver, iter));
            }
        };

        state.apply(direction, point);
        if let Some(snap) = snapshot {
            state.maybe_update_best(&point, &transform, snap);
        }
    }

    Ok(state.into_solution(Status::MaxIters, config.max_iters()))
}

// ============================================================================
// Eval + observe helper
// ============================================================================

enum EvalOutcome<I, O> {
    Continue {
        point: Point,
        snapshot: Snapshot<I, O>,
    },
    AssumeWorse,
    StopEarly,
}

/// Evaluate at `x`, emit event, and handle observer action.
fn eval_and_observe<M, P, Obs>(
    model: &M,
    problem: &P,
    x: f64,
    other: Point,
    observer: &mut Obs,
) -> Result<EvalOutcome<M::Input, M::Output>, Error>
where
    M: Model,
    P: OptimizationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'a> Observer<Event<'a, M, P>, Action>,
{
    match evaluate(model, problem, [x]) {
        Ok(eval) => {
            let point = Point::from(&eval);
            let event = Event::Evaluated {
                point,
                input: &eval.snapshot.input,
                output: &eval.snapshot.output,
                other,
            };
            match observer.observe(&event) {
                Some(Action::StopEarly) => Ok(EvalOutcome::StopEarly),
                Some(Action::AssumeWorse) => Ok(EvalOutcome::AssumeWorse),
                None => Ok(EvalOutcome::Continue {
                    point,
                    snapshot: eval.snapshot,
                }),
            }
        }
        Err(e) => {
            let action = Event::emit_failure(x, other, &e, observer);
            match action {
                Some(Action::StopEarly) => Ok(EvalOutcome::StopEarly),
                Some(Action::AssumeWorse) => Ok(EvalOutcome::AssumeWorse),
                None => Err(e.into()),
            }
        }
    }
}
