use twine_core::constraint::{Constrained, ConstraintResult, NonNegative};
use uom::si::f64::Pressure;

/// Pressure drops along the top and bottom streams.
///
/// Each pressure drop is defined as `p_inlet - p_outlet` for the stream and is
/// guaranteed to be non-negative.
///
/// The "top" and "bottom" labels refer to the physical stream assignment, not
/// necessarily the hot/cold side of the heat exchanger.
#[derive(Debug, Clone, Copy)]
pub struct PressureDrops {
    top: Pressure,
    bottom: Pressure,
}

impl PressureDrops {
    /// Construct validated pressure drops.
    ///
    /// # Errors
    ///
    /// Returns an error if either pressure drop is negative.
    pub fn new(top: Pressure, bottom: Pressure) -> ConstraintResult<Self> {
        Ok(Self::from_constrained(
            Constrained::<Pressure, NonNegative>::new(top)?,
            Constrained::<Pressure, NonNegative>::new(bottom)?,
        ))
    }

    /// Construct pressure drops from pre-validated values.
    #[must_use]
    pub fn from_constrained(
        top: Constrained<Pressure, NonNegative>,
        bottom: Constrained<Pressure, NonNegative>,
    ) -> Self {
        Self {
            top: top.into_inner(),
            bottom: bottom.into_inner(),
        }
    }

    /// Construct pressure drops without validation.
    ///
    /// # Warning
    ///
    /// The caller must ensure both pressure drops are non-negative.
    /// Violating this invariant will result in unexpected errors or panics.
    #[must_use]
    pub fn new_unchecked(top: Pressure, bottom: Pressure) -> Self {
        Self { top, bottom }
    }

    /// Returns the pressure drop of the top stream.
    #[must_use]
    pub fn top(&self) -> Pressure {
        self.top
    }

    /// Returns the pressure drop of the bottom stream.
    #[must_use]
    pub fn bottom(&self) -> Pressure {
        self.bottom
    }
}
