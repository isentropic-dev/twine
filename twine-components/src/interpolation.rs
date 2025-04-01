use ndarray::Array1;
use ninterp::{
    error::{InterpolateError, ValidateError},
    prelude::{Interp1DOwned, Interpolator},
    strategy,
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

/// Extrapolation strategy
///
/// Controls what happens if supplied interpolant point is outside the bounds of
/// the interpolation grid.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Extrapolate<T> {
    /// Evaluate beyond the limits of the interpolation grid.
    Enable,
    /// If point is beyond grid limits, return this value instead.
    Fill(T),
    /// Restrict interpolant point to the limits of the interpolation grid,
    /// using [`num_traits::clamp`].
    Clamp,
    /// Wrap around to other end of periodic data. Does NOT check that first and
    /// last values are equal.
    Wrap,
    /// Return an error when interpolant point is beyond the limits of the
    /// interpolation grid.
    #[default]
    Error,
}

impl<T> From<Extrapolate<T>> for ninterp::interpolator::Extrapolate<T> {
    fn from(value: Extrapolate<T>) -> Self {
        match value {
            Extrapolate::Enable => ninterp::interpolator::Extrapolate::Enable,
            Extrapolate::Fill(val) => ninterp::interpolator::Extrapolate::Fill(val),
            Extrapolate::Clamp => ninterp::interpolator::Extrapolate::Clamp,
            Extrapolate::Wrap => ninterp::interpolator::Extrapolate::Wrap,
            Extrapolate::Error => ninterp::interpolator::Extrapolate::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Linear,
    Nearest,
    LeftNearest,
    RightNearest,
}

pub enum Interp1D {
    Linear(Interp1DOwned<f64, strategy::Linear>),
}

impl Interp1D {
    #[allow(clippy::missing_errors_doc)]
    pub fn new<T: Into<Array1<f64>>>(
        x: T,
        f_x: T,
        strategy: Strategy,
        extrapolate: Extrapolate<f64>,
    ) -> Result<Self, InterpError> {
        match strategy {
            Strategy::Linear => Ok(Self::Linear(Interp1DOwned::new(
                x.into(),
                f_x.into(),
                strategy::Linear,
                extrapolate.into(),
            )?)),
            Strategy::Nearest => todo!(),
            Strategy::LeftNearest => todo!(),
            Strategy::RightNearest => todo!(),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn interpolate(&self, x: f64) -> Result<f64, InterpError> {
        match self {
            Interp1D::Linear(i) => i.interpolate(&[x]).map_err(Into::into),
        }
    }
}

impl Component for Interp1D {
    type Input = f64;
    type Output = f64;
    type Error = InterpError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.interpolate(input)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use twine_core::Component;

    use super::*;

    #[test]
    fn linear_1d_interp() {
        let linear = Interp1D::new(
            vec![0., 1., 2.],
            vec![0.0, 0.4, 0.8],
            Strategy::Linear,
            Extrapolate::Error,
        )
        .unwrap();

        assert_relative_eq!(linear.call(1.4).unwrap(), 0.56);
        assert!(linear.call(5.).is_err());
    }
}
