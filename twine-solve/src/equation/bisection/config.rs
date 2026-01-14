/// Configuration for the bisection solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub max_iters: usize,
    pub x_abs_tol: f64,
    pub x_rel_tol: f64,
    pub residual_tol: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_iters: 100,
            x_abs_tol: 1e-12,
            x_rel_tol: 1e-12,
            residual_tol: 1e-12,
        }
    }
}

impl Config {
    /// Validates that all tolerances are finite and non-negative.
    ///
    /// # Errors
    ///
    /// Returns an error if any tolerance is negative or non-finite.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.x_abs_tol.is_finite() || self.x_abs_tol < 0.0 {
            return Err("x_abs_tol must be finite and non-negative");
        }
        if !self.x_rel_tol.is_finite() || self.x_rel_tol < 0.0 {
            return Err("x_rel_tol must be finite and non-negative");
        }
        if !self.residual_tol.is_finite() || self.residual_tol < 0.0 {
            return Err("residual_tol must be finite and non-negative");
        }
        Ok(())
    }
}
