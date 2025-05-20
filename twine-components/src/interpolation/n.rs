use ndarray::{Array, Array1, IxDyn};
use ninterp::{
    prelude::{InterpNDOwned, Interpolator},
    strategy::enums::StrategyNDEnum,
};
use twine_core::Component;

use super::{error::InterpError, extrapolate::Extrapolate};

/// Interpolation strategies supported for N-dimensional interpolation.
///
/// Used with [`InterpND::new`] to control how values are estimated across an N-dimensional grid.
#[derive(Debug, Clone, Copy)]
pub enum StrategyND {
    /// Multilinear interpolation across N dimensions.
    Linear,

    /// Nearest-neighbor interpolation based on the closest N-dimensional grid point.
    Nearest,
}

pub struct InterpND(InterpNDOwned<f64, StrategyNDEnum>);

impl InterpND {
    /// Creates a new N-dimensional interpolator from axis-aligned grid coordinates, values, strategy, and extrapolation behavior.
    ///
    /// # Arguments
    ///
    /// * `grid` - A vector of 1D arrays, one per axis, containing sorted coordinate values.
    /// * `values` - An N-dimensional array of values corresponding to each coordinate combination.
    ///   The shape must match the lengths of the arrays in `grid`.
    /// * `strategy` - Interpolation strategy to use (e.g., linear or nearest).
    /// * `extrapolate` - Behavior to use when the input lies outside the bounds of the grid.
    ///
    /// # Errors
    ///
    /// Returns an error if the grid and value dimensions do not match, or if validation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use ndarray::array;
    /// use twine_components::interpolation::{InterpND, StrategyND, Extrapolate};
    /// use twine_core::Component;
    ///
    /// let interp = InterpND::new(
    ///     vec![array![1., 2.], array![1., 2.], array![1., 2.]],
    ///     array![
    ///         [ [0.6, 0.8], [0.8, 1.0] ],
    ///         [ [0.8, 1.0], [1.0, 1.2] ],
    ///     ].into_dyn(),
    ///     &StrategyND::Linear,
    ///     Extrapolate::Clamp,
    /// ).unwrap();
    ///
    /// let value = interp.call(vec![1.5, 1.5, 1.5]).unwrap();
    /// assert!(value == 0.9);
    /// ```
    pub fn new(
        grid: Vec<Array1<f64>>,
        values: Array<f64, IxDyn>,
        strategy: &StrategyND,
        extrapolate: Extrapolate<f64>,
    ) -> Result<Self, InterpError> {
        match strategy {
            StrategyND::Linear => Ok(Self(InterpNDOwned::new(
                grid,
                values,
                ninterp::strategy::Linear.into(),
                extrapolate.into(),
            )?)),
            StrategyND::Nearest => Ok(Self(InterpNDOwned::new(
                grid,
                values,
                ninterp::strategy::Nearest.into(),
                extrapolate.into(),
            )?)),
        }
    }
}

impl Component for InterpND {
    type Input = Vec<f64>;
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
        let test_cases = [(StrategyND::Linear, 0.9), (StrategyND::Nearest, 1.2)];

        for (strategy, expected) in test_cases {
            let interp = InterpND::new(
                vec![array![1., 2.], array![1., 2.], array![1., 2.]],
                array![[[0.6, 0.8], [0.8, 1.0],], [[0.8, 1.0], [1.0, 1.2]],].into_dyn(),
                &strategy,
                Extrapolate::Error,
            )
            .unwrap();

            let actual = interp.call(vec![1.5, 1.5, 1.5]).unwrap();
            assert!(
                approx::relative_eq!(actual, expected),
                "strategy {strategy:?} produced wrong result: got {actual}, expected {expected}"
            );
        }
    }
}
