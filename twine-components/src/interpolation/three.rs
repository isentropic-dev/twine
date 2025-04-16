use ndarray::{Array1, Array3};
use ninterp::{
    prelude::{Interp3DOwned, Interpolator},
    strategy::enums::Strategy3DEnum,
};
use twine_core::Component;

use super::{error::InterpError, extrapolate::Extrapolate};

/// Interpolation strategies supported for three-dimensional interpolation.
///
/// Used with [`Interp3D::new`] to control how values are estimated between grid points.
#[derive(Debug, Clone, Copy)]
pub enum Strategy3D {
    /// Trilinear interpolation between surrounding grid points.
    Linear,

    /// Nearest-neighbor interpolation based on the closest grid point.
    Nearest,
}

pub struct Interp3D(Interp3DOwned<f64, Strategy3DEnum>);

impl Interp3D {
    /// Creates a new 3D interpolator from grid coordinates, values, strategy, and extrapolation behavior.
    ///
    /// # Arguments
    ///
    /// * `x` - 1D array of grid coordinates along the x-axis.
    /// * `y` - 1D array of grid coordinates along the y-axis.
    /// * `z` - 1D array of grid coordinates along the z-axis.
    /// * `f_xyz` - 3D array of values corresponding to each `(x, y, z)` combination.  
    ///   Must have shape `(x.len(), y.len(), z.len())`.
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
    /// use twine_components::interpolation::{Interp3D, Strategy3D, Extrapolate};
    /// use twine_core::Component;
    ///
    /// let interp = Interp3D::new(
    ///     array![1., 2.],
    ///     array![1., 2.],
    ///     array![1., 2.],
    ///     array![
    ///         [ [0.6, 0.8], [0.8, 1.0] ],
    ///         [ [0.8, 1.0], [1.0, 1.2] ],
    ///     ],
    ///     &Strategy3D::Linear,
    ///     Extrapolate::Clamp,
    /// ).unwrap();
    ///
    /// let value = interp.call([1.5, 1.5, 1.5]).unwrap();
    /// assert!(value == 0.9);
    /// ```
    pub fn new(
        x: Array1<f64>,
        y: Array1<f64>,
        z: Array1<f64>,
        f_xyz: Array3<f64>,
        strategy: &Strategy3D,
        extrapolate: Extrapolate<f64>,
    ) -> Result<Self, InterpError> {
        match strategy {
            Strategy3D::Linear => Ok(Self(Interp3DOwned::new(
                x,
                y,
                z,
                f_xyz,
                ninterp::strategy::Linear.into(),
                extrapolate.into(),
            )?)),
            Strategy3D::Nearest => Ok(Self(Interp3DOwned::new(
                x,
                y,
                z,
                f_xyz,
                ninterp::strategy::Nearest.into(),
                extrapolate.into(),
            )?)),
        }
    }
}

impl Component for Interp3D {
    type Input = [f64; 3];
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
        let test_cases = [(Strategy3D::Linear, 0.9), (Strategy3D::Nearest, 1.2)];

        for (strategy, expected) in test_cases {
            let interp = Interp3D::new(
                array![1., 2.],
                array![1., 2.],
                array![1., 2.],
                array![[[0.6, 0.8], [0.8, 1.0],], [[0.8, 1.0], [1.0, 1.2]],],
                &strategy,
                Extrapolate::Error,
            )
            .unwrap();

            let actual = interp.call([1.5, 1.5, 1.5]).unwrap();
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
