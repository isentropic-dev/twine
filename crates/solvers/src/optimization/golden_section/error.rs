use crate::optimization::evaluate::EvalError;

/// Errors that can occur during golden section search.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("model error: {0}")]
    Model(Box<dyn std::error::Error + Send + Sync>),

    #[error("problem error: {0}")]
    Problem(Box<dyn std::error::Error + Send + Sync>),
}

impl<ME, PE> From<EvalError<ME, PE>> for Error
where
    ME: std::error::Error + Send + Sync + 'static,
    PE: std::error::Error + Send + Sync + 'static,
{
    fn from(err: EvalError<ME, PE>) -> Self {
        match err {
            EvalError::Model(e) => Self::Model(Box::new(e)),
            EvalError::Problem(e) => Self::Problem(Box::new(e)),
        }
    }
}
