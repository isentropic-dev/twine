use crate::equation::{EquationProblem, Evaluation, Observer, evaluate};
use twine_core::model::Model;

use super::{Action, Bracket, Event, decision::Decision};

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

    /// Evaluates the left endpoint and returns the observer decision.
    pub(crate) fn left_endpoint(&mut self, x: f64) -> EvalOutcome<M::Input, M::Output> {
        let result = evaluate(self.model, self.problem, [x]);
        let action = self.observer.observe(&Event::Left { x, result: &result });

        let (residual, mut eval) = match result {
            Ok(eval) => (Ok(eval.residuals[0]), Some(eval)),
            Err(error) => (Err(error.into()), None),
        };

        let decision = Decision::new(action, residual);

        if matches!(action, Some(Action::AssumeResidualSign(_))) {
            eval = None;
        }

        (eval, decision)
    }

    /// Evaluates the right endpoint and returns the observer decision.
    pub(crate) fn right_endpoint(&mut self, x: f64) -> EvalOutcome<M::Input, M::Output> {
        let result = evaluate(self.model, self.problem, [x]);
        let action = self.observer.observe(&Event::Right { x, result: &result });

        let (residual, mut eval) = match result {
            Ok(eval) => (Ok(eval.residuals[0]), Some(eval)),
            Err(error) => (Err(error.into()), None),
        };

        let decision = Decision::new(action, residual);

        if matches!(action, Some(Action::AssumeResidualSign(_))) {
            eval = None;
        }

        (eval, decision)
    }

    /// Evaluates the midpoint and returns the observer decision.
    pub(crate) fn midpoint(
        &mut self,
        x: f64,
        bracket: &Bracket,
    ) -> EvalOutcome<M::Input, M::Output> {
        let result = evaluate(self.model, self.problem, [x]);
        let action = self.observer.observe(&Event::Midpoint {
            x,
            bracket,
            result: &result,
        });

        let (residual, mut eval) = match result {
            Ok(eval) => (Ok(eval.residuals[0]), Some(eval)),
            Err(error) => (Err(error.into()), None),
        };

        let decision = Decision::new(action, residual);

        if matches!(action, Some(Action::AssumeResidualSign(_))) {
            eval = None;
        }

        (eval, decision)
    }
}
