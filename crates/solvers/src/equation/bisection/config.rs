use thiserror::Error;

/// Configuration for the bisection solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    pub max_iters: usize,
    pub x_abs_tol: f64,
    pub x_rel_tol: f64,
    pub residual_tol: f64,
}

/// Errors that can occur when validating a bisection config.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    #[error("x_abs_tol must be finite and non-negative")]
    XAbs,

    #[error("x_rel_tol must be finite and non-negative")]
    XRel,

    #[error("residual_tol must be finite and non-negative")]
    Residual,
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
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.x_abs_tol.is_finite() || self.x_abs_tol < 0.0 {
            return Err(ConfigError::XAbs);
        }
        if !self.x_rel_tol.is_finite() || self.x_rel_tol < 0.0 {
            return Err(ConfigError::XRel);
        }
        if !self.residual_tol.is_finite() || self.residual_tol < 0.0 {
            return Err(ConfigError::Residual);
        }
        Ok(())
    }
}
