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

    #[error("failed to compute input")]
    Input(#[source] Box<dyn StdError + Send + Sync>),

    #[error("model call failed")]
    Model(#[source] Box<dyn StdError + Send + Sync>),

    #[error("failed to compute residual")]
    Residual(#[source] Box<dyn StdError + Send + Sync>),
}

impl<IE, ME, RE> From<EvalError<IE, ME, RE>> for Error
where
    IE: StdError + Send + Sync + 'static,
    ME: StdError + Send + Sync + 'static,
    RE: StdError + Send + Sync + 'static,
{
    fn from(err: EvalError<IE, ME, RE>) -> Self {
        match err {
            EvalError::Input(e) => Self::Input(Box::new(e)),
            EvalError::Model(e) => Self::Model(Box::new(e)),
            EvalError::Residual(e) => Self::Residual(Box::new(e)),
        }
    }
}
