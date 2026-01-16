use crate::{
    equation::{EquationProblem, EvaluateResult, Evaluation, Observer, evaluate},
    model::Model,
};

use super::{Action, Bracket, Error, Event, decision::Decision};

type EvalOutcome<I, O> = (Option<Evaluation<I, O, 1>>, Decision);

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
    pub(crate) fn new(model: &'ctx M, problem: &'ctx P, observer: &'ctx mut Obs) -> Self {
        Self {
            model,
            problem,
            observer,
        }
    }

    /// Evaluates the left endpoint and returns the observer decision.
    pub(crate) fn left_endpoint(&mut self, x: f64) -> EvalOutcome<M::Input, M::Output> {
        let (result, action) = self.observe_left(x);
        let (residual, mut eval) = match result {
            Ok(eval) => (Ok(eval.residuals[0]), Some(eval)),
            Err(error) => (Err(Error::from(error)), None),
        };
        let decision = Decision::new(action, residual);
        if matches!(action, Some(Action::AssumeResidualSign(_))) {
            eval = None;
        }
        (eval, decision)
    }

    /// Evaluates the right endpoint and returns the observer decision.
    pub(crate) fn right_endpoint(&mut self, x: f64) -> EvalOutcome<M::Input, M::Output> {
        let (result, action) = self.observe_right(x);
        let (residual, mut eval) = match result {
            Ok(eval) => (Ok(eval.residuals[0]), Some(eval)),
            Err(error) => (Err(Error::from(error)), None),
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
        let (result, action) = self.observe_midpoint(x, bracket);
        let (residual, mut eval) = match result {
            Ok(eval) => (Ok(eval.residuals[0]), Some(eval)),
            Err(error) => (Err(Error::from(error)), None),
        };
        let decision = Decision::new(action, residual);
        if matches!(action, Some(Action::AssumeResidualSign(_))) {
            eval = None;
        }
        (eval, decision)
    }

    fn observe_left(&mut self, x: f64) -> (EvaluateResult<M, P, 1>, Option<Action>) {
        let model: &M = self.model;
        let problem: &P = self.problem;
        let observer: &mut Obs = self.observer;
        let result = evaluate(model, problem, [x]);
        let event = Event::Left { x, result: &result };
        let action = observer.observe(&event);
        (result, action)
    }

    fn observe_right(&mut self, x: f64) -> (EvaluateResult<M, P, 1>, Option<Action>) {
        let model: &M = self.model;
        let problem: &P = self.problem;
        let observer: &mut Obs = self.observer;
        let result = evaluate(model, problem, [x]);
        let event = Event::Right { x, result: &result };
        let action = observer.observe(&event);
        (result, action)
    }

    fn observe_midpoint(
        &mut self,
        x: f64,
        bracket: &Bracket,
    ) -> (EvaluateResult<M, P, 1>, Option<Action>) {
        let model: &M = self.model;
        let problem: &P = self.problem;
        let observer: &mut Obs = self.observer;
        let result = evaluate(model, problem, [x]);
        let event = Event::Midpoint {
            x,
            bracket,
            result: &result,
        };
        let action = observer.observe(&event);
        (result, action)
    }
}
