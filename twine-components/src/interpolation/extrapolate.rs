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
