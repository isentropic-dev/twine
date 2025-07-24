use twine_core::constraint::{Constrained, ConstraintError, StrictlyPositive};
use uom::{ConstZero, si::f64::MassRate};

use crate::{State, Stream};

/// Represents mass flow across a system boundary.
///
/// This enum represents flow direction relative to the system:
///
/// - `In`: Mass flows into the system (positive contribution).
/// - `Out`: Mass flows out of the system (negative contribution).
/// - `None`: No mass flow occurs.
///
/// The `In` variant holds a [`Stream`] with a thermodynamic [`State`].
/// The `Out` variant assumes mass exits at the system's current state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MassFlow<Fluid> {
    /// Mass flowing into the system.
    In(Stream<Fluid>),
    /// Mass flowing out of the system.
    Out(Constrained<MassRate, StrictlyPositive>),
    /// No mass flow occurs.
    None,
}

impl<Fluid> MassFlow<Fluid> {
    /// Creates a [`MassFlow::In`] representing mass flowing into the system with a defined state.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `mass_rate` is not strictly positive.
    pub fn incoming(mass_rate: MassRate, state: State<Fluid>) -> Result<Self, ConstraintError> {
        Ok(Self::In(Stream::new(mass_rate, state)?))
    }

    /// Creates a [`MassFlow::Out`] representing mass flowing out of the system.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `mass_rate` is not strictly positive.
    pub fn outgoing(mass_rate: MassRate) -> Result<Self, ConstraintError> {
        Ok(Self::Out(Constrained::new(mass_rate)?))
    }

    /// Creates a mass flow pair with equal inflow and outflow rates.
    ///
    /// Useful for modeling constant-mass systems where inflow is balanced by an
    /// equal outflow at the system's current state.
    pub fn balanced_pair(stream: Stream<Fluid>) -> (Self, Self) {
        let m_dot = stream.rate;
        (Self::In(stream), Self::Out(m_dot))
    }

    /// Returns the signed mass flow rate.
    ///
    /// - Positive for mass flowing into the system.
    /// - Negative for mass flowing out of the system.
    /// - Zero if no mass flow.
    #[must_use]
    pub fn signed_mass_rate(&self) -> MassRate {
        match self {
            Self::In(Stream { rate, .. }) => rate.into_inner(),
            Self::Out(rate) => -rate.into_inner(),
            Self::None => MassRate::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, MassRate, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        mass_rate::kilogram_per_second,
        thermodynamic_temperature::kelvin,
    };

    use crate::{State, fluid::Air};

    fn default_state() -> State<Air> {
        State {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            fluid: Air,
        }
    }

    #[test]
    fn incoming_mass_is_positive() {
        let m_dot = MassRate::new::<kilogram_per_second>(1.5);
        let flow = MassFlow::incoming(m_dot, default_state()).unwrap();
        assert!(matches!(flow, MassFlow::In(_)));
        assert_relative_eq!(flow.signed_mass_rate().get::<kilogram_per_second>(), 1.5);
    }

    #[test]
    fn outgoing_mass_is_negative() {
        let m_dot = MassRate::new::<kilogram_per_second>(0.8);
        let flow: MassFlow<Air> = MassFlow::outgoing(m_dot).unwrap();
        assert!(matches!(flow, MassFlow::Out(_)));
        assert_relative_eq!(flow.signed_mass_rate().get::<kilogram_per_second>(), -0.8);
    }

    #[test]
    fn none_mass_is_zero() {
        let flow: MassFlow<Air> = MassFlow::None;
        assert_relative_eq!(flow.signed_mass_rate().get::<kilogram_per_second>(), 0.0);
    }

    #[test]
    fn rejects_negative_incoming() {
        let m_dot = MassRate::new::<kilogram_per_second>(-1.0);
        assert!(MassFlow::incoming(m_dot, default_state()).is_err());
    }

    #[test]
    fn rejects_zero_incoming() {
        let m_dot = MassRate::new::<kilogram_per_second>(0.0);
        assert!(MassFlow::incoming(m_dot, default_state()).is_err());
    }

    #[test]
    fn rejects_negative_outgoing() {
        let m_dot = MassRate::new::<kilogram_per_second>(-0.5);
        assert!(MassFlow::<Air>::outgoing(m_dot).is_err());
    }

    #[test]
    fn rejects_zero_outgoing() {
        let m_dot = MassRate::new::<kilogram_per_second>(0.0);
        assert!(MassFlow::<Air>::outgoing(m_dot).is_err());
    }

    #[test]
    fn balanced_pair_produces_equal_and_opposite_flows() {
        let (mass_flow_in, mass_flow_out) = MassFlow::balanced_pair(
            Stream::new(MassRate::new::<kilogram_per_second>(2.0), default_state()).unwrap(),
        );

        assert!(matches!(mass_flow_in, MassFlow::In(_)));
        assert!(matches!(mass_flow_out, MassFlow::Out(_)));

        let m_dot_in = mass_flow_in.signed_mass_rate();
        let m_dot_out = mass_flow_out.signed_mass_rate();

        assert_relative_eq!(m_dot_in.get::<kilogram_per_second>(), 2.0);
        assert_relative_eq!(m_dot_out.get::<kilogram_per_second>(), -2.0);
        assert_eq!(m_dot_in + m_dot_out, MassRate::ZERO);
    }
}
