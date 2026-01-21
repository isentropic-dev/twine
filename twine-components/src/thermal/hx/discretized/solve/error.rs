use thiserror::Error;
use uom::{
    ConstZero,
    si::f64::{Power, TemperatureInterval, ThermodynamicTemperature},
};

use crate::thermal::hx::discretized::{HeatTransferRate, MinDeltaT};

use super::Resolved;

/// Errors that can occur while solving a discretized heat exchanger.
#[derive(Debug, Error)]
pub enum SolveError {
    /// A Second Law violation occurred.
    ///
    /// This includes cases where computed heat transfer is NaN or invalid,
    /// which typically indicates non-physical thermodynamic states.
    ///
    /// Note: Any of the reported values may be NaN if states or calculations
    /// produced non-numeric results.
    #[error("second law violation: min_delta_t={min_delta_t:?}")]
    SecondLawViolation {
        /// Top stream outlet temperature, if resolved.
        top_outlet_temp: Option<ThermodynamicTemperature>,

        /// Bottom stream outlet temperature, if resolved.
        bottom_outlet_temp: Option<ThermodynamicTemperature>,

        /// Heat transfer rate.
        q_dot: Power,

        /// Minimum temperature difference (`T_hot` - `T_cold`) encountered.
        /// Negative indicates violation.
        min_delta_t: TemperatureInterval,

        /// Node index where violation occurred, if detected during discretization.
        /// `None` if detected during outlet resolution.
        violation_node: Option<usize>,
    },

    /// A thermodynamic model operation failed.
    ///
    /// This failure can be from property evaluation or state construction.
    #[error("thermodynamic model failed: {context}")]
    ThermoModelFailed {
        /// Operation context for the thermodynamic model failure.
        context: String,

        /// Underlying thermodynamic model error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl SolveError {
    /// Creates a thermo model failure error with context.
    pub(super) fn thermo_failed(
        context: impl Into<String>,
        err: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ThermoModelFailed {
            context: context.into(),
            source: Box::new(err),
        }
    }

    /// Checks second-law constraints for the resolved solution.
    ///
    /// Validates that heat flows in the thermodynamically correct direction
    /// (hot to cold) and that no temperature crossover occurs (negative ΔT).
    ///
    /// # Errors
    ///
    /// Returns [`SolveError::SecondLawViolation`] if constraints are violated.
    pub(super) fn check_second_law<TopFluid, BottomFluid>(
        resolved: &Resolved<TopFluid, BottomFluid>,
        min_delta_t: MinDeltaT,
    ) -> Result<(), Self> {
        if resolved.q_dot == HeatTransferRate::None {
            return Ok(());
        }

        let top_is_hot = resolved.top.inlet.temperature >= resolved.bottom.inlet.temperature;
        let direction_mismatch = match resolved.q_dot {
            HeatTransferRate::TopToBottom(_) => !top_is_hot,
            HeatTransferRate::BottomToTop(_) => top_is_hot,
            HeatTransferRate::None => unreachable!(),
        };

        let negative_delta_t = min_delta_t.value < TemperatureInterval::ZERO;

        if direction_mismatch || negative_delta_t {
            return Err(Self::SecondLawViolation {
                top_outlet_temp: Some(resolved.top.outlet.temperature),
                bottom_outlet_temp: Some(resolved.bottom.outlet.temperature),
                q_dot: resolved.q_dot.signed_top_to_bottom(),
                min_delta_t: min_delta_t.value,
                violation_node: Some(min_delta_t.node),
            });
        }

        Ok(())
    }
}
