use std::cmp::Ordering;

use twine_core::constraint::{Constrained, ConstraintError, StrictlyPositive};
use uom::{ConstZero, si::f64::Power};

/// Represents heat flow across a system boundary.
///
/// This type encodes heat flow direction and magnitude using a sign convention
/// consistent with energy balances:
///
/// - `In`: Heat flows into the system (positive contribution).
/// - `Out`: Heat flows out of the system (negative contribution).
/// - `None`: No heat flow occurs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatFlow {
    /// Heat flowing into the system.
    In(Constrained<Power, StrictlyPositive>),
    /// Heat flowing out of the system.
    Out(Constrained<Power, StrictlyPositive>),
    /// No heat flow occurs.
    None,
}

impl HeatFlow {
    /// Creates a [`HeatFlow::In`] representing heat flowing into the system.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `heat_rate` is not strictly positive.
    pub fn incoming(heat_rate: Power) -> Result<Self, ConstraintError> {
        Ok(Self::In(Constrained::new(heat_rate)?))
    }

    /// Creates a [`HeatFlow::Out`] representing heat flowing out of the system.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `heat_rate` is not strictly positive.
    pub fn outgoing(heat_rate: Power) -> Result<Self, ConstraintError> {
        Ok(Self::Out(Constrained::new(heat_rate)?))
    }

    /// Creates a [`HeatFlow`] from a signed heat flow rate.
    ///
    /// - Positive values indicate heat flowing into the system.
    /// - Negative values indicate heat flowing out of the system.
    /// - Zero indicates no heat flow.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError::NotANumber`] if the value is not finite.
    pub fn from_signed(heat_rate: Power) -> Result<Self, ConstraintError> {
        match heat_rate.partial_cmp(&Power::ZERO) {
            Some(Ordering::Greater) => Self::incoming(heat_rate),
            Some(Ordering::Less) => Self::outgoing(-heat_rate),
            Some(Ordering::Equal) => Ok(Self::None),
            None => Err(ConstraintError::NotANumber),
        }
    }

    /// Returns the signed heat flow rate.
    ///
    /// - Positive for heat flowing into the system.
    /// - Negative for heat flowing out of the system.
    /// - Zero if no heat flow.
    #[must_use]
    pub fn signed(&self) -> Power {
        match self {
            Self::In(heat_rate) => heat_rate.into_inner(),
            Self::Out(heat_rate) => -heat_rate.into_inner(),
            Self::None => Power::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{f64::Power, power::watt};

    #[test]
    fn incoming_is_positive() {
        let q_dot = Power::new::<watt>(100.0);
        let flow = HeatFlow::incoming(q_dot).unwrap();
        assert!(matches!(flow, HeatFlow::In(_)));
        assert_relative_eq!(flow.signed().get::<watt>(), 100.0);
    }

    #[test]
    fn outgoing_is_negative() {
        let q_dot = Power::new::<watt>(200.0);
        let flow = HeatFlow::outgoing(q_dot).unwrap();
        assert!(matches!(flow, HeatFlow::Out(_)));
        assert_relative_eq!(flow.signed().get::<watt>(), -200.0);
    }

    #[test]
    fn none_is_zero() {
        let flow = HeatFlow::None;
        assert_relative_eq!(flow.signed().get::<watt>(), 0.0);
    }

    #[test]
    fn from_signed_heat_rate_classifies_correctly() {
        let in_flow = HeatFlow::from_signed(Power::new::<watt>(50.0)).unwrap();
        let out_flow = HeatFlow::from_signed(Power::new::<watt>(-75.0)).unwrap();
        let none_flow = HeatFlow::from_signed(Power::new::<watt>(0.0)).unwrap();

        assert!(matches!(in_flow, HeatFlow::In(_)));
        assert!(matches!(out_flow, HeatFlow::Out(_)));
        assert!(matches!(none_flow, HeatFlow::None));
    }

    #[test]
    fn rejects_nan_input() {
        let q_dot = Power::new::<watt>(f64::NAN);
        let result = HeatFlow::from_signed(q_dot);
        assert!(matches!(result, Err(ConstraintError::NotANumber)));
    }

    #[test]
    fn rejects_negative_incoming() {
        let q_dot = Power::new::<watt>(-1.0);
        assert!(HeatFlow::incoming(q_dot).is_err());
    }

    #[test]
    fn rejects_zero_incoming() {
        let q_dot = Power::new::<watt>(0.0);
        assert!(HeatFlow::incoming(q_dot).is_err());
    }
}
