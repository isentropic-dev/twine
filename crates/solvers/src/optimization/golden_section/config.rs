use thiserror::Error;

/// Configuration for the golden section solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    max_iters: usize,
    x_abs_tol: f64,
    x_rel_tol: f64,
}

/// Errors that can occur when validating a golden section solver config.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    #[error("x_abs_tol must be finite and non-negative")]
    XAbs,

    #[error("x_rel_tol must be finite and non-negative")]
    XRel,
}

impl Default for Config {
    fn default() -> Self {
        // Known-good values, unwrap is safe
        Self::new(100, 1e-12, 1e-12).unwrap()
    }
}

impl Config {
    /// Creates a new config with validated tolerances.
    ///
    /// # Errors
    ///
    /// Returns an error if any tolerance is negative or non-finite.
    pub fn new(max_iters: usize, x_abs_tol: f64, x_rel_tol: f64) -> Result<Self, ConfigError> {
        if !x_abs_tol.is_finite() || x_abs_tol < 0.0 {
            return Err(ConfigError::XAbs);
        }
        if !x_rel_tol.is_finite() || x_rel_tol < 0.0 {
            return Err(ConfigError::XRel);
        }

        Ok(Self {
            max_iters,
            x_abs_tol,
            x_rel_tol,
        })
    }

    /// Returns the maximum number of shrink iterations.
    #[must_use]
    pub fn max_iters(&self) -> usize {
        self.max_iters
    }

    /// Returns the absolute tolerance for x convergence.
    #[must_use]
    pub fn x_abs_tol(&self) -> f64 {
        self.x_abs_tol
    }

    /// Returns the relative tolerance for x convergence.
    #[must_use]
    pub fn x_rel_tol(&self) -> f64 {
        self.x_rel_tol
    }
}
