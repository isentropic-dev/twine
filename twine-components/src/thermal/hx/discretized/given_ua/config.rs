use twine_solve::equation::bisection;
use uom::si::{
    f64::{TemperatureInterval, ThermalConductance},
    temperature_interval::kelvin as delta_kelvin,
    thermal_conductance::watt_per_kelvin,
};

/// Solver configuration for iterative UA matching.
#[derive(Debug, Clone, Copy)]
pub struct GivenUaConfig {
    /// Maximum iteration count for the bisection solve.
    pub max_iters: usize,

    /// Absolute tolerance for the outlet temperature search variable.
    pub temp_tol: TemperatureInterval,

    /// Absolute tolerance for the UA residual (achieved - target).
    pub ua_tol: ThermalConductance,
}

impl Default for GivenUaConfig {
    fn default() -> Self {
        Self {
            max_iters: 100,
            temp_tol: TemperatureInterval::new::<delta_kelvin>(1e-12),
            ua_tol: ThermalConductance::new::<watt_per_kelvin>(1e-12),
        }
    }
}

impl GivenUaConfig {
    /// Converts this configuration into a bisection solver configuration.
    pub(super) fn bisection(&self) -> bisection::Config {
        bisection::Config {
            max_iters: self.max_iters,
            x_abs_tol: self.temp_tol.get::<delta_kelvin>(),
            x_rel_tol: 0.0,
            residual_tol: self.ua_tol.get::<watt_per_kelvin>(),
        }
    }
}
