/// Errors that can occur during golden section search.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("model error: {0}")]
    Model(Box<dyn std::error::Error + Send + Sync>),

    #[error("problem error: {0}")]
    Problem(Box<dyn std::error::Error + Send + Sync>),
}
