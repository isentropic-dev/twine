//! Error types for discretized heat exchanger solving.

use thiserror::Error;
use uom::si::f64::{Power, TemperatureInterval, ThermodynamicTemperature};

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
    /// Create a thermo model failure error with context.
    pub(super) fn thermo_failed(
        context: impl Into<String>,
        err: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ThermoModelFailed {
            context: context.into(),
            source: Box::new(err),
        }
    }
}
