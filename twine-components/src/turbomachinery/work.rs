//! Work types for turbomachinery models.
//!
//! Work in turbomachinery can be reported with different sign conventions.
//! Twine turbomachinery components instead expose work as a non-negative quantity,
//! with direction encoded in the type.
//!
//! - [`CompressionWork`] represents the shaft work input required by a compressor.
//! - [`ExpansionWork`] represents the shaft work output produced by a turbine.

use twine_core::constraint::{Constrained, ConstraintError, NonNegative};
use twine_thermo::units::SpecificEnthalpy;

/// Specific shaft work for compression.
///
/// The inner value is a [`SpecificEnthalpy`] that is guaranteed to be non-negative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompressionWork(SpecificEnthalpy);

impl CompressionWork {
    /// Returns zero compression work.
    #[must_use]
    pub fn zero() -> Self {
        Self::from_constrained(NonNegative::zero())
    }

    /// Constructs a [`CompressionWork`] if `work >= 0`.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `work` is negative or not comparable (e.g., NaN).
    pub fn new(work: SpecificEnthalpy) -> Result<Self, ConstraintError> {
        let work = NonNegative::new(work)?;
        Ok(Self::from_constrained(work))
    }

    /// Creates a new [`CompressionWork`] from a pre-validated non-negative work value.
    #[must_use]
    pub fn from_constrained(work: Constrained<SpecificEnthalpy, NonNegative>) -> Self {
        Self(work.into_inner())
    }

    /// Returns the underlying specific work quantity.
    #[must_use]
    pub fn quantity(&self) -> SpecificEnthalpy {
        self.0
    }
}

/// Specific shaft work for expansion.
///
/// The inner value is a [`SpecificEnthalpy`] that is guaranteed to be non-negative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExpansionWork(SpecificEnthalpy);

impl ExpansionWork {
    /// Returns zero expansion work.
    #[must_use]
    pub fn zero() -> Self {
        Self::from_constrained(NonNegative::zero())
    }

    /// Constructs an [`ExpansionWork`] if `work >= 0`.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `work` is negative or not comparable (e.g., NaN).
    pub fn new(work: SpecificEnthalpy) -> Result<Self, ConstraintError> {
        let work = NonNegative::new(work)?;
        Ok(Self::from_constrained(work))
    }

    /// Creates a new [`ExpansionWork`] from a pre-validated non-negative work value.
    #[must_use]
    pub fn from_constrained(work: Constrained<SpecificEnthalpy, NonNegative>) -> Self {
        Self(work.into_inner())
    }

    /// Returns the underlying specific work quantity.
    #[must_use]
    pub fn quantity(&self) -> SpecificEnthalpy {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use twine_core::constraint::NonNegative;
    use uom::si::{
        energy::joule,
        f64::{Energy, Mass},
        mass::kilogram,
    };

    /// Constructs a specific enthalpy in SI units (J/kg).
    fn enth_si(value: f64) -> SpecificEnthalpy {
        Energy::new::<joule>(value) / Mass::new::<kilogram>(1.0)
    }

    #[test]
    fn compression_work_rejects_negative() {
        let negative = enth_si(-1.0);
        assert!(CompressionWork::new(negative).is_err());
    }

    #[test]
    fn compression_work_from_constrained_quantity_roundtrip() {
        let value = enth_si(42.0);
        let constrained = NonNegative::new(value).unwrap();
        let work = CompressionWork::from_constrained(constrained);
        assert_eq!(work.quantity(), value);
    }

    #[test]
    fn expansion_work_rejects_negative() {
        let negative = enth_si(-1.0);
        assert!(ExpansionWork::new(negative).is_err());
    }

    #[test]
    fn expansion_work_from_constrained() {
        let value = enth_si(5.0);
        let constrained = NonNegative::new(value).unwrap();
        let work = ExpansionWork::from_constrained(constrained);
        assert_eq!(work.quantity(), value);
    }
}
