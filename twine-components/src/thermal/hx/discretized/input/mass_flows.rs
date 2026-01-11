use twine_core::constraint::{Constrained, ConstraintResult, StrictlyPositive};
use uom::si::f64::MassRate;

/// Mass flow rates for the top and bottom streams.
///
/// The "top" and "bottom" labels refer to the physical stream assignment,
/// not necessarily the hot/cold side of the heat exchanger.
///
/// Each mass flow rate is guaranteed to be strictly positive.
#[derive(Debug, Clone, Copy)]
pub struct MassFlows {
    top: MassRate,
    bottom: MassRate,
}

impl MassFlows {
    /// Construct validated mass flows.
    ///
    /// # Errors
    ///
    /// Returns an error if either flow rate is not strictly positive.
    pub fn new(top: MassRate, bottom: MassRate) -> ConstraintResult<Self> {
        Ok(Self::from_constrained(
            Constrained::<MassRate, StrictlyPositive>::new(top)?,
            Constrained::<MassRate, StrictlyPositive>::new(bottom)?,
        ))
    }

    /// Construct mass flows from pre-validated values.
    #[must_use]
    pub fn from_constrained(
        top: Constrained<MassRate, StrictlyPositive>,
        bottom: Constrained<MassRate, StrictlyPositive>,
    ) -> Self {
        Self {
            top: top.into_inner(),
            bottom: bottom.into_inner(),
        }
    }

    /// Construct mass flows without validation.
    ///
    /// # Warning
    ///
    /// The caller must ensure both flow rates are strictly positive.
    /// Violating this invariant will result in unexpected errors or panics.
    #[must_use]
    pub fn new_unchecked(top: MassRate, bottom: MassRate) -> Self {
        Self { top, bottom }
    }

    /// Returns the mass flow rate of the top stream.
    #[must_use]
    pub fn top(&self) -> MassRate {
        self.top
    }

    /// Returns the mass flow rate of the bottom stream.
    #[must_use]
    pub fn bottom(&self) -> MassRate {
        self.bottom
    }
}
