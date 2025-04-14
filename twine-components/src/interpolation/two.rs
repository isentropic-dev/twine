use ndarray::{Array1, Array2};
use ninterp::{
    prelude::{Interp2DOwned, Interpolator},
    strategy::enums::Strategy2DEnum,
};
use twine_core::Component;

use super::{error::InterpError, extrapolate::Extrapolate};

#[derive(Debug, Clone, Copy)]
pub enum Strategy2D {
    Linear,
    Nearest,
}

pub struct Interp2D(Interp2DOwned<f64, Strategy2DEnum>);

impl Interp2D {
    pub fn new(
        x: Array1<f64>,
        y: Array1<f64>,
        f_xy: Array2<f64>,
        strategy: &Strategy2D,
        extrapolate: Extrapolate<f64>,
    ) -> Result<Self, InterpError> {
        match strategy {
            Strategy2D::Linear => Ok(Self(Interp2DOwned::new(
                x.into(),
                y.into(),
                f_xy.into(),
                ninterp::strategy::Linear.into(),
                extrapolate.into(),
            )?)),
            Strategy2D::Nearest => Ok(Self(Interp2DOwned::new(
                x.into(),
                y.into(),
                f_xy.into(),
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
