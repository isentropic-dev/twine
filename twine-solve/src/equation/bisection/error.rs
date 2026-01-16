use std::error::Error as StdError;

use thiserror::Error;

use crate::equation::EvalError;

/// Errors that can occur during bisection solving.
#[derive(Debug, Error)]
pub enum Error {
    #[error("bracket has zero width: left and right are both {value}")]
    ZeroWidthBracket { value: f64 },

    #[error("bracket contains non-finite value: {value}")]
    NonFiniteBracket { value: f64 },

    #[error("no root in bracket: f({left})={left_residual}, f({right})={right_residual}")]
    NoBracket {
        left: f64,
        right: f64,
        left_residual: f64,
        right_residual: f64,
    },

    #[error("invalid config: {reason}")]
    InvalidConfig { reason: &'static str },

    #[error("failed to compute input")]
    Input(#[source] Box<dyn StdError + Send + Sync>),

    #[error("model call failed")]
    Model(#[source] Box<dyn StdError + Send + Sync>),

    #[error("failed to compute residual")]
    Residual(#[source] Box<dyn StdError + Send + Sync>),

    #[error("non-finite residual {residual} at x = {x}")]
    NonFiniteResidual { x: f64, residual: f64 },
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
