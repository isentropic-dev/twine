use std::ops::Deref;

use thiserror::Error;
use uom::{
    Conversion,
    si::{f64::MassRate, mass_rate},
};

/// A mass flow rate guaranteed to be non-negative.
///
/// This type is typically used to enforce the physical constraint that mass
/// flow through a system must be zero or positive.
///
/// Use [`PositiveMassRate::new`] to construct a new instance with unit safety,
/// or [`TryFrom`] to fallibly convert from an existing [`MassRate`].
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct PositiveMassRate(MassRate);

impl PositiveMassRate {
    /// Constructs a new `PositiveMassRate` from a numeric value and a unit.
    ///
    /// This method ensures that the provided value is non-negative.
    ///
    /// The unit type must come from the [`uom::si::mass_rate`] module and is
    /// specified as a type parameter.
    ///
    /// # Parameters
    ///
    /// - `value`: The numeric mass flow rate value.
    /// - `U`: The unit type (e.g., [`kilogram_per_second`], [`pound_per_hour`]).
    ///
    /// # Returns
    ///
    /// - `Ok(PositiveMassRate)` if the value is greater than or equal to zero.
    /// - `Err(PositiveMassRateError::NegativeRate)` if the value is negative.
    ///
    /// # Errors
    ///
    /// This function returns [`PositiveMassRateError::NegativeRate`] if the
    /// input value is less than zero.
    /// The error includes the rejected value in kilograms per second.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use uom::si::mass_rate::kilogram_per_second;
    ///
    /// let m_dot = PositiveMassRate::new::<kilogram_per_second>(2.0)?;
    /// ```
    pub fn new<U>(value: f64) -> Result<Self, PositiveMassRateError>
    where
        U: mass_rate::Unit + Conversion<f64, T = f64>,
    {
        let rate = MassRate::new::<U>(value);
        Self::from_mass_rate(rate)
    }

    /// Attempts to construct a `PositiveMassRate` from an existing [`MassRate`].
    ///
    /// # Errors
    ///
    /// Returns [`PositiveMassRateError::NegativeRate`] if the provided rate is
    /// less than zero.
    pub fn from_mass_rate(rate: MassRate) -> Result<Self, PositiveMassRateError> {
        if rate.value >= 0.0 {
            Ok(Self(rate))
        } else {
            Err(PositiveMassRateError::NegativeRate(
                rate.get::<mass_rate::kilogram_per_second>(),
            ))
        }
    }

    /// Returns a `PositiveMassRate` with a value of zero.
    ///
    /// Equivalent to `PositiveMassRate::default()`.
    #[must_use]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Consumes the wrapper and returns the underlying [`MassRate`] value.
    ///
    /// This is useful when you need to extract the wrapped value for further
    /// calculations or conversions.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> MassRate {
        self.0
    }
}

/// Dereferences to the inner [`MassRate`] value.
///
/// This allows you to call methods and access units on `PositiveMassRate` as if
/// it were a `MassRate`, without requiring manual extraction.
///
/// # Example
///
/// ```ignore
/// use uom::si::mass_rate::kilogram_per_second;
///
/// let m_dot = PositiveMassRate::new::<kilogram_per_second>(1.0)?;
/// let rho = /* some density value */;
/// let v_dot = *m_dot / rho;
/// ```
impl Deref for PositiveMassRate {
    type Target = MassRate;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<MassRate> for PositiveMassRate {
    type Error = PositiveMassRateError;

    fn try_from(rate: MassRate) -> Result<Self, Self::Error> {
        PositiveMassRate::from_mass_rate(rate)
    }
}

/// Errors that can occur when constructing a [`PositiveMassRate`].
#[derive(Debug, Clone, Copy, PartialEq, Error)]
#[non_exhaustive]
pub enum PositiveMassRateError {
    /// Returned when the provided mass rate is less than zero.
    #[error("Mass flow rate must be non-negative, got {0} kg/s")]
    NegativeRate(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::mass_rate::{kilogram_per_second, pound_per_hour};

    #[test]
    fn creating_positive_mass_rates() {
        // Positive mass flow rate should succeed.
        let result = PositiveMassRate::new::<kilogram_per_second>(5.0);
        assert!(result.is_ok());

        // Zero mass flow rate should succeed.
        let result = PositiveMassRate::new::<pound_per_hour>(0.0);
        assert!(result.is_ok());

        // Negative mass flow rate should fail.
        let result = PositiveMassRate::new::<kilogram_per_second>(-2.0);
        assert_eq!(result, Err(PositiveMassRateError::NegativeRate(-2.0)));
    }

    #[test]
    fn try_into_positive_mass_rate() {
        let m_dot = MassRate::new::<kilogram_per_second>(5.0);

        let result: Result<PositiveMassRate, _> = m_dot.try_into();
        assert!(result.is_ok());

        let result: Result<PositiveMassRate, _> = (-m_dot).try_into();
        assert_eq!(result, Err(PositiveMassRateError::NegativeRate(-5.0)));
    }
}
