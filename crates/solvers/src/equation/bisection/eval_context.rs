use twine_core::{EquationProblem, Model, Observer};

use crate::equation::{Evaluation, bracket::Bracket, evaluate};

use super::{Action, Decision, Event, Point};

type EvalOutcome<I, O> = (Option<Evaluation<I, O, 1>>, Decision);

/// Bundles evaluation and observation for a single bisection solve.
///
/// This keeps event emission and action handling in one place while leaving the
/// solver loop to focus on control flow.
///
/// Each evaluation is split into a residual result used for decisions and an
/// optional evaluation used for best tracking.
/// When the observer assumes a residual sign, the evaluation is dropped so it
/// does not update the best solution.
pub(crate) struct EvalContext<'ctx, M, P, Obs> {
    model: &'ctx M,
    problem: &'ctx P,
    observer: &'ctx mut Obs,
}

impl<'ctx, M, P, Obs> EvalContext<'ctx, M, P, Obs>
where
    M: Model,
    M::Input: Clone,
    M::Output: Clone,
    P: EquationProblem<1, Input = M::Input, Output = M::Output>,
    Obs: for<'evt> Observer<Event<'evt, M, P>, Action>,
{
    /// Creates a new evaluation context.
    pub(crate) fn new(model: &'ctx M, problem: &'ctx P, observer: &'ctx mut Obs) -> Self {
        Self {
            model,
            problem,
            observer,
        }
    }

    /// Evaluates the midpoint, emits an event, and returns the outcome.
    pub(crate) fn midpoint(
        &mut self,
        x: f64,
        bracket: &Bracket,
    ) -> EvalOutcome<M::Input, M::Output> {
        match evaluate(self.model, self.problem, [x]) {
            Ok(eval) => {
                let point = Point::from(&eval);
                let event = Event::Evaluated {
                    point,
                    input: &eval.snapshot.input,
                    output: &eval.snapshot.output,
                    bracket,
                };
                let action = self.observer.observe(&event);
                let decision = Decision::new(action, Ok(point.residual));

                let kept_eval = if matches!(action, Some(Action::AssumeResidualSign(_))) {
                    None
                } else {
                    Some(eval)
                };

                (kept_eval, decision)
            }
            Err(error) => {
                let action = Event::emit_failure(x, bracket, &error, self.observer);
                let decision = Decision::new(action, Err(error.into()));
                (None, decision)
            }
        }
    }
}
