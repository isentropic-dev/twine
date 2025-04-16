use ninterp::error::{InterpolateError, ValidateError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterpError {
    #[error(transparent)]
    Validation(#[from] ValidateError),
    #[error(transparent)]
    Interpolation(#[from] InterpolateError),
}
