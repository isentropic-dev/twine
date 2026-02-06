use super::{Action, Error, Sign};

/// Control flow outcomes for a single evaluation.
#[derive(Debug)]
pub(crate) enum Decision {
    Continue(Sign),
    StopEarly,
    Error(Error),
}

impl Decision {
    /// Computes a decision from observer action and residual outcome.
    pub(crate) fn new(action: Option<Action>, residual_result: Result<f64, Error>) -> Self {
        match action {
            Some(Action::AssumeResidualSign(sign)) => Decision::Continue(sign),
            Some(Action::StopEarly) => Decision::StopEarly,
            None => match residual_result {
                Ok(value) => Decision::Continue(Sign::of(value)),
                Err(error) => Decision::Error(error),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err() -> Error {
        Error::Problem(Box::new(std::fmt::Error))
    }

    #[test]
    fn uses_residual_sign_when_no_action() {
        let decision = Decision::new(None, Ok(-0.1));
        assert!(matches!(decision, Decision::Continue(Sign::Negative)));
    }

    #[test]
    fn assume_residual_sign_works_with_failed_eval() {
        let decision = Decision::new(Some(Action::assume_positive()), Err(err()));
        assert!(matches!(decision, Decision::Continue(Sign::Positive)));
    }

    #[test]
    fn assume_residual_sign_ignores_residual_value() {
        let decision = Decision::new(Some(Action::assume_negative()), Ok(0.1));
        assert!(matches!(decision, Decision::Continue(Sign::Negative)));
    }

    #[test]
    fn stop_early_ignores_eval() {
        let decision = Decision::new(Some(Action::StopEarly), Ok(1.0));
        assert!(matches!(decision, Decision::StopEarly));

        let decision = Decision::new(Some(Action::StopEarly), Err(err()));
        assert!(matches!(decision, Decision::StopEarly));
    }

    #[test]
    fn returns_error_when_eval_fails() {
        let decision = Decision::new(None, Err(err()));
        assert!(matches!(decision, Decision::Error(_)));
    }
}
