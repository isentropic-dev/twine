use std::error::Error as StdError;

/// Errors that can occur during Euler integration.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("model error: {0}")]
    Model(#[source] Box<dyn StdError + Send + Sync>),

    #[error("problem error: {0}")]
    Problem(#[source] Box<dyn StdError + Send + Sync>),
}

impl Error {
    pub(crate) fn model<E: StdError + Send + Sync + 'static>(err: E) -> Self {
        Self::Model(Box::new(err))
    }

    pub(crate) fn problem<E: StdError + Send + Sync + 'static>(err: E) -> Self {
        Self::Problem(Box::new(err))
    }
}
