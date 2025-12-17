use std::error::Error as StdError;

use thiserror::Error;
use twine_thermo::{
    PropertyError, State,
    units::{SpecificEnthalpy, SpecificEntropy},
};
use uom::si::f64::Pressure;

use crate::turbomachinery::work::ExpansionWork;

/// Result of a turbine calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpansionResult<Fluid> {
    /// Outlet state returned by the thermodynamic model.
    ///
    /// For models that construct states by inverse-solving or interpolation, this
    /// returned state may be an approximate solution to the requested target.
    pub outlet: State<Fluid>,

    /// Produced specific shaft work.
    pub work: ExpansionWork,
}

/// Errors that may occur when calling a turbine model.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ExpansionError<Fluid> {
    /// The requested outlet pressure is greater than the inlet pressure.
    #[error("outlet pressure must not be greater than inlet (p_in={p_in:?}, p_out={p_out:?})")]
    OutletPressureGreaterThanInlet { p_in: Pressure, p_out: Pressure },

    /// The computed target work is negative.
    ///
    /// This is uncommon and is usually caused by very small pressure ratios
    /// combined with thermodynamic model numerical effects.
    ///
    /// Includes the outlet state returned by the thermodynamic model so callers can
    /// decide how to recover without recomputing it.
    #[error("computed expansion work is non-physical (raw_work={raw_work:?})")]
    NonPhysicalWork {
        outlet: State<Fluid>,
        raw_work: SpecificEnthalpy,
    },

    /// A thermodynamic model operation failed.
    ///
    /// This failure can be from property evaluation or state construction.
    #[error("thermodynamic model failed: {context}")]
    ThermodynamicModelFailed {
        context: String,
        #[source]
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl<Fluid> ExpansionError<Fluid> {
    /// Wrap a failure to evaluate pressure at the inlet.
    pub(crate) fn inlet_pressure_failed(source: PropertyError) -> Self {
        Self::ThermodynamicModelFailed {
            context: "pressure(inlet)".to_string(),
            source: Box::new(source),
        }
    }

    /// Wrap a failure to evaluate enthalpy at the inlet.
    pub(crate) fn inlet_enthalpy_failed(source: PropertyError) -> Self {
        Self::ThermodynamicModelFailed {
            context: "enthalpy(inlet)".to_string(),
            source: Box::new(source),
        }
    }

    /// Wrap a failure to evaluate entropy at the inlet.
    pub(crate) fn inlet_entropy_failed(source: PropertyError) -> Self {
        Self::ThermodynamicModelFailed {
            context: "entropy(inlet)".to_string(),
            source: Box::new(source),
        }
    }

    /// Wrap a failure to evaluate enthalpy at the ideal outlet.
    pub(crate) fn ideal_outlet_enthalpy_failed(source: PropertyError) -> Self {
        Self::ThermodynamicModelFailed {
            context: "enthalpy(ideal outlet)".to_string(),
            source: Box::new(source),
        }
    }

    /// Wrap a failure to construct the ideal outlet state from `(p_out, s_in)`.
    pub(crate) fn ideal_outlet_state_from_pressure_entropy_failed<E>(
        p_out: Pressure,
        s_in: SpecificEntropy,
        source: E,
    ) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::ThermodynamicModelFailed {
            context: format!("ideal_outlet_state_from(p_out={p_out:?}, s_in={s_in:?})"),
            source: Box::new(source),
        }
    }

    /// Wrap a failure to construct the outlet state from `(p_out, h_out_target)`.
    pub(crate) fn outlet_state_from_pressure_enthalpy_failed<E>(
        p_out: Pressure,
        h_out_target: SpecificEnthalpy,
        source: E,
    ) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::ThermodynamicModelFailed {
            context: format!("outlet_state_from(p_out={p_out:?}, h_out_target={h_out_target:?})"),
            source: Box::new(source),
        }
    }
}
