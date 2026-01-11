use std::cmp::Ordering;

use twine_core::constraint::{Constrained, ConstraintError, ConstraintResult, StrictlyPositive};
use uom::{ConstZero, si::f64::Power};

/// Directional heat transfer rate for a discretized heat exchanger.
///
/// This type represents direction explicitly, without relying on sign.
/// When a nonzero heat transfer is present, the stored `Power` value is
/// guaranteed to be strictly positive.
///
/// The "top" and "bottom" labels refer to the physical stream assignment,
/// not necessarily the hot/cold side of the heat exchanger.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatTransferRate {
    /// Heat flows from the top stream to the bottom stream.
    TopToBottom(Power),
    /// Heat flows from the bottom stream to the top stream.
    BottomToTop(Power),
    /// No heat transfer occurs.
    None,
}

impl HeatTransferRate {
    /// Construct a heat transfer rate from the top stream to the bottom stream.
    ///
    /// # Errors
    ///
    /// Returns an error if `q_dot` is not strictly positive.
    pub fn top_to_bottom(q_dot: Power) -> ConstraintResult<Self> {
        Ok(Self::top_to_bottom_from_constrained(Constrained::<
            Power,
            StrictlyPositive,
        >::new(q_dot)?))
    }

    /// Construct a heat transfer rate from the top stream to the bottom stream
    /// using a pre-validated power.
    #[must_use]
    pub fn top_to_bottom_from_constrained(q_dot: Constrained<Power, StrictlyPositive>) -> Self {
        Self::TopToBottom(q_dot.into_inner())
    }

    /// Construct a heat transfer rate from the bottom stream to the top stream.
    ///
    /// # Errors
    ///
    /// Returns an error if `q_dot` is not strictly positive.
    pub fn bottom_to_top(q_dot: Power) -> Result<Self, ConstraintError> {
        Ok(Self::bottom_to_top_from_constrained(Constrained::<
            Power,
            StrictlyPositive,
        >::new(q_dot)?))
    }

    /// Construct a heat transfer rate from the bottom stream to the top stream
    /// using a pre-validated power.
    #[must_use]
    pub fn bottom_to_top_from_constrained(q_dot: Constrained<Power, StrictlyPositive>) -> Self {
        Self::BottomToTop(q_dot.into_inner())
    }

    /// Construct a heat transfer rate from a signed quantity.
    ///
    /// Positive values mean heat flows from the top stream to the bottom stream.
    /// Negative values mean heat flows from the bottom stream to the top stream.
    /// Zero indicates no heat transfer.
    ///
    /// # Errors
    ///
    /// Returns an error if `q_dot` is not a number.
    pub fn from_signed_top_to_bottom(q_dot: Power) -> Result<Self, ConstraintError> {
        match q_dot.partial_cmp(&Power::ZERO) {
            Some(Ordering::Greater) => Ok(Self::TopToBottom(q_dot)),
            Some(Ordering::Less) => Ok(Self::BottomToTop(-q_dot)),
            Some(Ordering::Equal) => Ok(Self::None),
            None => Err(ConstraintError::NotANumber),
        }
    }

    /// Returns the signed heat transfer rate (positive means top-to-bottom).
    ///
    /// The sign convention matches [`HeatTransferRate::from_signed_top_to_bottom`].
    #[must_use]
    pub fn signed_top_to_bottom(&self) -> Power {
        match *self {
            Self::TopToBottom(q) => q,
            Self::BottomToTop(q) => -q,
            Self::None => Power::ZERO,
        }
    }

    /// Returns the non-negative heat transfer magnitude.
    #[must_use]
    pub fn magnitude(&self) -> Power {
        match *self {
            Self::TopToBottom(q) | Self::BottomToTop(q) => q,
            Self::None => Power::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::power::watt;

    #[test]
    fn from_signed_maps_directions() {
        let q_pos = Power::new::<watt>(10.0);
        let q_neg = Power::new::<watt>(-5.0);

        assert_eq!(
            HeatTransferRate::from_signed_top_to_bottom(q_pos).unwrap(),
            HeatTransferRate::TopToBottom(q_pos)
        );
        assert_eq!(
            HeatTransferRate::from_signed_top_to_bottom(q_neg).unwrap(),
            HeatTransferRate::BottomToTop(Power::new::<watt>(5.0))
        );
        assert_eq!(
            HeatTransferRate::from_signed_top_to_bottom(Power::ZERO).unwrap(),
            HeatTransferRate::None
        );
    }

    #[test]
    fn signed_and_magnitude_conventions() {
        let q = Power::new::<watt>(12.0);
        let top_to_bottom = HeatTransferRate::TopToBottom(q);
        let bottom_to_top = HeatTransferRate::BottomToTop(q);

        assert_eq!(top_to_bottom.signed_top_to_bottom(), q);
        assert_eq!(bottom_to_top.signed_top_to_bottom(), -q);
        assert_eq!(top_to_bottom.magnitude(), q);
        assert_eq!(bottom_to_top.magnitude(), q);
        assert_eq!(HeatTransferRate::None.magnitude(), Power::ZERO);
    }
}
