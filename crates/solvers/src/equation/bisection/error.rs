use std::error::Error as StdError;

use thiserror::Error;

use crate::equation::EvalError;

use super::{bracket::BracketError, config::ConfigError};

/// Errors that can occur during bisection solving.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid bracket: {0}")]
    InvalidBracket(#[from] BracketError),

    #[error("invalid config: {0}")]
    InvalidConfig(#[from] ConfigError),

    #[error("no successful evaluations")]
    NoSuccessfulEvaluation,

    #[error("problem error")]
    Problem(#[source] Box<dyn StdError + Send + Sync>),

    #[error("model call failed")]
    Model(#[source] Box<dyn StdError + Send + Sync>),
}

impl<ME, PE> From<EvalError<ME, PE>> for Error
where
    ME: StdError + Send + Sync + 'static,
    PE: StdError + Send + Sync + 'static,
{
    fn from(err: EvalError<ME, PE>) -> Self {
        match err {
            EvalError::Model(e) => Self::Model(Box::new(e)),
            EvalError::Problem(e) => Self::Problem(Box::new(e)),
        }
    }
}
