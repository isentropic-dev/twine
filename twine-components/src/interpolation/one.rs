use ndarray::Array1;
use ninterp::{
    prelude::{Interp1DOwned, Interpolator},
    strategy::enums::Strategy1DEnum,
};
use twine_core::Component;

use super::{error::InterpError, extrapolate::Extrapolate};

#[derive(Debug, Clone, Copy)]
pub enum Strategy1D {
    Linear,
    Nearest,
    LeftNearest,
    RightNearest,
}

pub struct Interp1D(Interp1DOwned<f64, Strategy1DEnum>);

impl Interp1D {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(
        x: Array1<f64>,
        f_x: Array1<f64>,
        strategy: &Strategy1D,
        extrapolate: Extrapolate<f64>,
    ) -> Result<Self, InterpError> {
        match strategy {
            Strategy1D::Linear => Ok(Self(Interp1DOwned::new(
                x.into(),
                f_x.into(),
                ninterp::strategy::Linear.into(),
                extrapolate.into(),
            )?)),
            Strategy1D::Nearest => Ok(Self(Interp1DOwned::new(
                x.into(),
                f_x.into(),
                ninterp::strategy::Nearest.into(),
                extrapolate.into(),
            )?)),
            Strategy1D::LeftNearest => Ok(Self(Interp1DOwned::new(
                x.into(),
                f_x.into(),
                ninterp::strategy::LeftNearest.into(),
                extrapolate.into(),
            )?)),
            Strategy1D::RightNearest => Ok(Self(Interp1DOwned::new(
                x.into(),
                f_x.into(),
                ninterp::strategy::RightNearest.into(),
                extrapolate.into(),
            )?)),
        }
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
    use ndarray::array;
    use twine_core::Component;

    use super::*;

    #[test]
    fn strategy_interp_matches_expected_value() {
        let test_cases = [
            (Strategy1D::Linear, 0.56),
            (Strategy1D::Nearest, 0.4),
            (Strategy1D::LeftNearest, 0.4),
            (Strategy1D::RightNearest, 0.8),
        ];

        for (strategy, expected) in test_cases {
            let interp = Interp1D::new(
                array![0., 1., 2.],
                array![0.0, 0.4, 0.8],
                &strategy,
                Extrapolate::Error,
            )
            .unwrap();

            let actual = interp.call(1.4).unwrap();
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
