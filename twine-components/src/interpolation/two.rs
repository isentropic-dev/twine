use ndarray::{Array1, Array2};
use ninterp::{
    prelude::{Interp2DOwned, Interpolator},
    strategy::enums::Strategy2DEnum,
};
use twine_core::Component;

use super::{error::InterpError, extrapolate::Extrapolate};

/// Interpolation strategies supported for two-dimensional interpolation.
///
/// Used with [`Interp2D::new`] to control how values are estimated between grid points.
#[derive(Debug, Clone, Copy)]
pub enum Strategy2D {
    /// Linear interpolation using bilinear weighting.
    Linear,

    /// Nearest-neighbor interpolation based on closest grid point.
    Nearest,
}

pub struct Interp2D(Interp2DOwned<f64, Strategy2DEnum>);

impl Interp2D {
    /// Creates a new 2D interpolator from grid coordinates, values, strategy, and extrapolation behavior.
    ///
    /// # Arguments
    ///
    /// * `x` - 1D array of grid coordinates along the x-axis.
    /// * `y` - 1D array of grid coordinates along the y-axis.
    /// * `f_xy` - 2D array of values corresponding to each `(x, y)` pair.  
    ///   Must have shape `(x.len(), y.len())`.
    /// * `strategy` - Interpolation strategy to use (e.g., linear or nearest).
    /// * `extrapolate` - Behavior to use when the input is outside the bounds of the grid.
    ///
    /// # Errors
    ///
    /// Returns an error if the input arrays have incompatible shapes or other validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ndarray::array;
    /// use twine_components::interpolation::{Interp2D, Strategy2D, Extrapolate};
    /// use twine_core::Component;
    ///
    /// let interp = Interp2D::new(
    ///     array![0., 1., 2.],
    ///     array![0., 1., 2.],
    ///     array![
    ///         [0.0, 0.4, 0.8],
    ///         [0.2, 0.6, 1.0],
    ///         [0.4, 0.8, 1.2],
    ///     ],
    ///     &Strategy2D::Linear,
    ///     Extrapolate::Clamp,
    /// ).unwrap();
    ///
    /// let value = interp.call([1.5, 1.5]).unwrap();
    /// assert!(value == 0.9);
    /// ```
    pub fn new(
        x: Array1<f64>,
        y: Array1<f64>,
        f_xy: Array2<f64>,
        strategy: &Strategy2D,
        extrapolate: Extrapolate<f64>,
    ) -> Result<Self, InterpError> {
        match strategy {
            Strategy2D::Linear => Ok(Self(Interp2DOwned::new(
                x,
                y,
                f_xy,
                ninterp::strategy::Linear.into(),
                extrapolate.into(),
            )?)),
            Strategy2D::Nearest => Ok(Self(Interp2DOwned::new(
                x,
                y,
                f_xy,
                ninterp::strategy::Nearest.into(),
                extrapolate.into(),
            )?)),
        }
    }
}

impl Component for Interp2D {
    type Input = [f64; 2];
    type Output = f64;
    type Error = InterpError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.0.interpolate(&input).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use ndarray::array;
    use twine_core::Component;

    use super::*;

    #[test]
    fn strategy_interp_matches_expected_value() {
        let test_cases = [(Strategy2D::Linear, 0.9), (Strategy2D::Nearest, 1.2)];

        for (strategy, expected) in test_cases {
            let interp = Interp2D::new(
                array![0., 1., 2.],
                array![0.0, 1., 2.],
                array![[0.0, 0.4, 0.8], [0.2, 0.6, 1.0], [0.4, 0.8, 1.2]],
                &strategy,
                Extrapolate::Error,
            )
            .unwrap();

            let actual = interp.call([1.5, 1.5]).unwrap();
            assert!(
                approx::relative_eq!(actual, expected),
                "strategy {:?} produced wrong result: got {}, expected {}",
                strategy,
                actual,
                expected
            );
        }
    }
}
