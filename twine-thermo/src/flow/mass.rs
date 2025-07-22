use twine_core::constraint::{Constrained, ConstraintError, StrictlyPositive};
use uom::{
    ConstZero,
    si::f64::{MassRate, Power},
};

use crate::{PropertyError, State, model::ThermodynamicProperties, units::SpecificEnthalpy};

/// Represents mass flow across a system boundary.
///
/// This type encodes mass flow direction and magnitude using a sign convention
/// consistent with energy balances:
///
/// - `In`: Mass flows into the system (positive contribution, includes inflow state).
/// - `Out`: Mass flows out of the system (negative contribution, uses current system state).
/// - `None`: No mass flow occurs.
///
/// Inflow includes a thermodynamic [`State`] used to evaluate properties like enthalpy.
/// Outflow assumes the flow exits at the control volume's internal state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MassFlow<Fluid> {
    /// Mass flowing into the system with a defined thermodynamic state.
    In(Constrained<MassRate, StrictlyPositive>, State<Fluid>),
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
        Ok(Self::In(Constrained::new(mass_rate)?, state))
    }

    /// Creates a [`MassFlow::Out`] representing mass flowing out of the system.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `mass_rate` is not strictly positive.
    pub fn outgoing(mass_rate: MassRate) -> Result<Self, ConstraintError> {
        Ok(Self::Out(Constrained::new(mass_rate)?))
    }

    /// Returns the signed mass flow rate.
    ///
    /// - Positive for mass flowing into the system.
    /// - Negative for mass flowing out of the system.
    /// - Zero if no mass flow.
    #[must_use]
    pub fn signed_mass_rate(&self) -> MassRate {
        match self {
            Self::In(mass_rate, _) => mass_rate.into_inner(),
            Self::Out(mass_rate) => -mass_rate.into_inner(),
            Self::None => MassRate::ZERO,
        }
    }

    /// Returns the signed enthalpy flow rate associated with this mass flow.
    ///
    /// - Positive for enthalpy carried into the system.
    /// - Negative for enthalpy carried out of the system.
    /// - Zero if no flow.
    ///
    /// # Parameters
    ///
    /// - `model`: The thermodynamic model used to evaluate specific enthalpy.
    /// - `internal_enthalpy`: The enthalpy of the system's internal state (used for outflow).
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if the model fails to compute enthalpy for an inflow.
    pub fn signed_enthalpy_rate<Model>(
        &self,
        model: &Model,
        internal_enthalpy: SpecificEnthalpy,
    ) -> Result<Power, PropertyError>
    where
        Model: ThermodynamicProperties<Fluid>,
    {
        match self {
            Self::In(mass_rate, state) => Ok(mass_rate.into_inner() * model.enthalpy(state)?),
            Self::Out(mass_rate) => Ok(-mass_rate.into_inner() * internal_enthalpy),
            Self::None => Ok(Power::ZERO),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, MassRate, Power, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        mass_rate::kilogram_per_second,
        thermodynamic_temperature::kelvin,
    };

    use crate::{
        State,
        fluid::Air,
        model::{ThermodynamicProperties, ideal_gas::IdealGas},
    };

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
        assert!(matches!(flow, MassFlow::In(_, _)));
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
    fn enthalpy_rate_in_and_out() {
        let state_cv = State {
            temperature: ThermodynamicTemperature::new::<kelvin>(400.0),
            ..default_state()
        };
        let h_cv = IdealGas.enthalpy(&state_cv).unwrap();

        let m_dot = MassRate::new::<kilogram_per_second>(2.0);
        let inflow = MassFlow::incoming(m_dot, default_state()).unwrap();
        let outflow: MassFlow<Air> = MassFlow::outgoing(m_dot).unwrap();

        let h_dot_in = inflow.signed_enthalpy_rate(&IdealGas, h_cv).unwrap();
        let h_dot_out = outflow.signed_enthalpy_rate(&IdealGas, h_cv).unwrap();
        assert!(h_dot_in > Power::ZERO);
        assert!(h_dot_out < Power::ZERO);
        assert!(h_dot_out.abs() > h_dot_in.abs());
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
}
