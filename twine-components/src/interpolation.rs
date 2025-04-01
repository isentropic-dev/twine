use ninterp::{
    error::{InterpolateError, ValidateError},
    prelude::{Interp1DOwned, Interpolator},
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

macro_rules! define_interpolators {
    (
        $enum_name:ident, $interp_name:ident, $interp_type:ident, $input:ty, $new_fn:ident, $call_fn:ident;
        $($variant:ident => $strategy:path),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $($variant),+
        }

        pub enum $interp_name {
            $($variant($interp_type<$input, $strategy>)),+
        }

        impl $interp_name {
            #[allow(clippy::missing_errors_doc)]
            pub fn $new_fn<T: Into<ndarray::Array1<$input>>>(
                x: T,
                f_x: T,
                strategy: $enum_name,
                extrapolate: Extrapolate<$input>,
            ) -> Result<Self, InterpError> {
                match strategy {
                    $($enum_name::$variant => Ok(Self::$variant(
                        $interp_type::new(x.into(), f_x.into(), $strategy, extrapolate.into())?
                    )),)+
                }
            }

            #[allow(clippy::missing_errors_doc)]
            pub fn $call_fn(&self, input: $input) -> Result<$input, InterpError> {
                match self {
                    $(Self::$variant(i) => i.interpolate(&[input]).map_err(Into::into),)+
                }
            }
        }

        impl Component for $interp_name {
            type Input = $input;
            type Output = $input;
            type Error = InterpError;

            fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
                self.$call_fn(input)
            }
        }
    };
}

define_interpolators!(
    Strategy1D, Interp1D, Interp1DOwned, f64, new, interpolate;
    Linear => ninterp::strategy::Linear,
    Nearest => ninterp::strategy::Nearest,
    LeftNearest => ninterp::strategy::LeftNearest,
    RightNearest => ninterp::strategy::RightNearest,
);

#[cfg(test)]
mod tests {
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
                vec![0., 1., 2.],
                vec![0.0, 0.4, 0.8],
                strategy,
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
