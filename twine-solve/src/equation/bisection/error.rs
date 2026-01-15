use std::error::Error as StdError;

use thiserror::Error;

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

    #[error("model evaluation failed")]
    Model(#[source] Box<dyn StdError + Send + Sync>),

    #[error("failed to compute residual")]
    Residual(#[source] Box<dyn StdError + Send + Sync>),

    #[error("non-finite residual {residual} at x = {x}")]
    NonFiniteResidual { x: f64, residual: f64 },
}

impl Error {
    /// Wraps an error that occurred while computing the input.
    pub fn input(err: impl StdError + Send + Sync + 'static) -> Self {
        Self::Input(Box::new(err))
    }

    /// Wraps an error that occurred during model evaluation.
    pub fn model(err: impl StdError + Send + Sync + 'static) -> Self {
        Self::Model(Box::new(err))
    }

    /// Wraps an error that occurred while computing the residual.
    pub fn residual(err: impl StdError + Send + Sync + 'static) -> Self {
        Self::Residual(Box::new(err))
    }
}
