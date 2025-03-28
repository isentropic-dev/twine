use ndarray::Array1;
use ninterp::{
    error::{InterpolateError, ValidateError},
    prelude::{Extrapolate, Interp1DOwned, Interpolator},
    strategy::{self},
};
use thiserror::Error;
use twine_core::Component;

#[derive(Error, Debug)]
pub enum InterpError {
    #[error(transparent)]
    Validation(#[from] ValidateError),
    #[error(transparent)]
    Interpolation(#[from] InterpolateError),
}

pub struct Interp1D(Interp1DOwned<f64, strategy::Linear>);

impl Interp1D {
    #[allow(clippy::missing_errors_doc)]
    pub fn new<T: Into<Array1<f64>>>(x: T, f_x: T) -> Result<Self, InterpError> {
        Ok(Self(Interp1DOwned::new(
            x.into(),
            f_x.into(),
            strategy::Linear,
            Extrapolate::Error,
        )?))
    }
}

impl Component for Interp1D {
    type Input = f64;
    type Output = f64;
    type Error = InterpError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.0.interpolate(&[input]).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use twine_core::Component;

    use super::Interp1D;

    #[test]
    fn linear_1d_interp() {
        let linear = Interp1D::new(vec![0., 1., 2.], vec![0.0, 0.4, 0.8]).unwrap();

        let output = linear.call(1.4).unwrap();

        assert_relative_eq!(output, 0.56);
    }
}
