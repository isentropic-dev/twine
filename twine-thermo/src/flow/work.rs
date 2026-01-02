use std::cmp::Ordering;

use twine_core::constraint::{Constrained, ConstraintError, StrictlyPositive};
use uom::{ConstZero, si::f64::Power};

/// Represents work flow across a system boundary.
///
/// This enum represents flow direction relative to the system:
///
/// - `In`: Work flows into the system (positive contribution, work done on the system).
/// - `Out`: Work flows out of the system (negative contribution, work done by the system).
/// - `None`: No work flow occurs.
///
/// **Note:** Some thermodynamic texts define work as positive when **done by**
/// the system (i.e., flowing *out*).
/// This crate uses the opposite convention: **positive = into the system**,
/// for consistency with [`HeatFlow`](crate::HeatFlow) and
/// [`MassFlow`](crate::MassFlow) sign conventions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkFlow {
    /// Work flowing into the system.
    In(Constrained<Power, StrictlyPositive>),
    /// Work flowing out of the system.
    Out(Constrained<Power, StrictlyPositive>),
    /// No work flow occurs.
    None,
}

impl WorkFlow {
    /// Creates a [`WorkFlow::In`] representing work flowing into the system.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `work_rate` is not strictly positive.
    pub fn incoming(work_rate: Power) -> Result<Self, ConstraintError> {
        Ok(Self::In(Constrained::new(work_rate)?))
    }

    /// Creates a [`WorkFlow::Out`] representing work flowing out of the system.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `work_rate` is not strictly positive.
    pub fn outgoing(work_rate: Power) -> Result<Self, ConstraintError> {
        Ok(Self::Out(Constrained::new(work_rate)?))
    }

    /// Creates a [`WorkFlow`] from a signed work flow rate.
    ///
    /// - Positive values indicate work flowing into the system.
    /// - Negative values indicate work flowing out of the system.
    /// - Zero indicates no work flow.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError::NotANumber`] if the value is not finite.
    pub fn from_signed(work_rate: Power) -> Result<Self, ConstraintError> {
        match work_rate.partial_cmp(&Power::ZERO) {
            Some(Ordering::Greater) => Self::incoming(work_rate),
            Some(Ordering::Less) => Self::outgoing(-work_rate),
            Some(Ordering::Equal) => Ok(Self::None),
            None => Err(ConstraintError::NotANumber),
        }
    }

    /// Returns the signed work flow rate.
    ///
    /// - Positive for work flowing into the system.
    /// - Negative for work flowing out of the system.
    /// - Zero if no work flow.
    #[must_use]
    pub fn signed(&self) -> Power {
        match self {
            Self::In(work_rate) => work_rate.into_inner(),
            Self::Out(work_rate) => -work_rate.into_inner(),
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
        let w_dot = Power::new::<watt>(150.0);
        let flow = WorkFlow::incoming(w_dot).unwrap();
        assert!(matches!(flow, WorkFlow::In(_)));
        assert_relative_eq!(flow.signed().get::<watt>(), 150.0);
    }

    #[test]
    fn outgoing_is_negative() {
        let w_dot = Power::new::<watt>(250.0);
        let flow = WorkFlow::outgoing(w_dot).unwrap();
        assert!(matches!(flow, WorkFlow::Out(_)));
        assert_relative_eq!(flow.signed().get::<watt>(), -250.0);
    }

    #[test]
    fn none_is_zero() {
        let flow = WorkFlow::None;
        assert_relative_eq!(flow.signed().get::<watt>(), 0.0);
    }

    #[test]
    fn from_signed_work_rate_classifies_correctly() {
        let in_flow = WorkFlow::from_signed(Power::new::<watt>(75.0)).unwrap();
        let out_flow = WorkFlow::from_signed(Power::new::<watt>(-50.0)).unwrap();
        let none_flow = WorkFlow::from_signed(Power::new::<watt>(0.0)).unwrap();

        assert!(matches!(in_flow, WorkFlow::In(_)));
        assert!(matches!(out_flow, WorkFlow::Out(_)));
        assert!(matches!(none_flow, WorkFlow::None));
    }

    #[test]
    fn rejects_nan_input() {
        let w_dot = Power::new::<watt>(f64::NAN);
        let result = WorkFlow::from_signed(w_dot);
        assert!(matches!(result, Err(ConstraintError::NotANumber)));
    }

    #[test]
    fn rejects_negative_incoming() {
        let w_dot = Power::new::<watt>(-1.0);
        assert!(WorkFlow::incoming(w_dot).is_err());
    }

    #[test]
    fn rejects_zero_incoming() {
        let w_dot = Power::new::<watt>(0.0);
        assert!(WorkFlow::incoming(w_dot).is_err());
    }
}
