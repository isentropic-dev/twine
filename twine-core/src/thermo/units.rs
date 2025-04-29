use std::ops::Deref;

use thiserror::Error;
use uom::{
    si::{
        f64::{MassRate, TemperatureInterval, ThermodynamicTemperature},
        mass_rate::kilogram_per_second,
        temperature_interval::kelvin as delta_kelvin,
        thermodynamic_temperature::kelvin as abs_kelvin,
        Quantity, ISQ, SI,
    },
    typenum::{N1, N2, N3, P1, P2, Z0},
};

/// Specific gas constant, J/kg·K in SI.
pub type SpecificGasConstant = Quantity<ISQ<P2, Z0, N2, Z0, N1, Z0, Z0>, SI<f64>, f64>;

/// Specific enthalpy, J/kg in SI.
pub type SpecificEnthalpy = Quantity<ISQ<P2, Z0, N2, Z0, Z0, Z0, Z0>, SI<f64>, f64>;

/// Specific entropy, J/kg·K is SI.
pub type SpecificEntropy = Quantity<ISQ<P2, Z0, N2, Z0, N1, Z0, Z0>, SI<f64>, f64>;

/// Specific internal energy, J/kg in SI.
pub type SpecificInternalEnergy = Quantity<ISQ<P2, Z0, N2, Z0, Z0, Z0, Z0>, SI<f64>, f64>;

/// Temperature rate of change, K/s in SI.
pub type TemperatureRate = Quantity<ISQ<Z0, Z0, N1, Z0, P1, Z0, Z0>, SI<f64>, f64>;

/// U-value, or overall heat transfer coefficient, W/m²·K in SI.
///
/// Typically used to model heat loss through a surface with `Q = U⋅A⋅ΔT`.
pub type UValue = Quantity<ISQ<Z0, P1, N3, Z0, N1, Z0, Z0>, SI<f64>, f64>;

/// Mass flow rate constrained to be non-negative.
///
/// Typically used to enforce the physical constraint that mass flow through a
/// system must be positive or zero.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PositiveMassRate(MassRate);

impl PositiveMassRate {
    /// Creates a new `PositiveMassRate` if the value is non-negative.
    ///
    /// # Errors
    ///
    /// Returns a [`MassRateError`] if the provided `MassRate` is negative.
    pub fn new(rate: MassRate) -> Result<Self, MassRateError> {
        if rate.value >= 0.0 {
            Ok(Self(rate))
        } else {
            Err(MassRateError::NegativeRate(
                rate.get::<kilogram_per_second>(),
            ))
        }
    }

    /// Consumes the wrapper and returns the inner `MassRate`.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> MassRate {
        self.0
    }
}

impl Deref for PositiveMassRate {
    type Target = MassRate;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Errors that can occur when creating a [`PositiveMassRate`].
#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum MassRateError {
    /// The provided mass rate was negative.
    #[error("Mass flow rate must be non-negative, got {0} kg/s")]
    NegativeRate(f64),
}

/// Computes the difference between two temperatures.
///
/// A `TemperatureInterval` (representing a temperature change) is a distinct
/// quantity from a `ThermodynamicTemperature` (representing an absolute
/// temperature). This function provides a safe and unit-consistent way to
/// compute the difference.
///
/// The input temperatures may be expressed in any supported units (Kelvin,
/// Celsius, Fahrenheit, etc.). Internally, they are converted to kelvin (K)
/// for computation.
///
/// The returned `TemperatureInterval` represents the signed temperature
/// difference, and can be displayed or accessed in any compatible units.
///
/// The sign of the result is meaningful:
/// - Positive if temperature increases from `from` to `to`.
/// - Negative if temperature decreases from `from` to `to`.
///
/// # Parameters
///
/// - `from`: The starting temperature value for the comparison.
/// - `to`: The ending temperature value for the comparison.
///
/// # Returns
///
/// A `TemperatureInterval` representing the signed difference `to - from`.
#[inline]
#[must_use]
pub fn temperature_difference(
    from: ThermodynamicTemperature,
    to: ThermodynamicTemperature,
) -> TemperatureInterval {
    TemperatureInterval::new::<delta_kelvin>(to.get::<abs_kelvin>() - from.get::<abs_kelvin>())
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        mass_rate::{kilogram_per_second, pound_per_hour},
        temperature_interval::{
            degree_celsius as delta_celsius, degree_rankine as delta_rankine,
            kelvin as delta_kelvin,
        },
        thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin},
    };

    #[test]
    fn positive_mass_rate_creation() {
        // Positive mass flow rate should succeed.
        let positive_rate = MassRate::new::<kilogram_per_second>(5.0);
        let result = PositiveMassRate::new(positive_rate);
        assert!(result.is_ok());

        // Zero mass flow rate should succeed.
        let zero_rate = MassRate::new::<pound_per_hour>(0.0);
        let result = PositiveMassRate::new(zero_rate);
        assert!(result.is_ok());

        // Negative mass flow rate should fail.
        let negative_rate = MassRate::new::<kilogram_per_second>(-2.0);
        let result = PositiveMassRate::new(negative_rate);
        assert_eq!(result, Err(MassRateError::NegativeRate(-2.0)));
    }

    #[test]
    fn temperature_difference_sign_and_magnitude() {
        // Positive temperature change.
        let from = ThermodynamicTemperature::new::<kelvin>(300.0);
        let to = ThermodynamicTemperature::new::<kelvin>(310.0);
        let delta = temperature_difference(from, to);
        assert_relative_eq!(delta.get::<delta_kelvin>(), 10.0);
        assert_relative_eq!(delta.get::<delta_rankine>(), 18.0);

        // Negative temperature change.
        let from = ThermodynamicTemperature::new::<kelvin>(310.0);
        let to = ThermodynamicTemperature::new::<kelvin>(300.0);
        let delta = temperature_difference(from, to);
        assert_relative_eq!(delta.get::<delta_kelvin>(), -10.0);

        // No difference in temperature.
        let from = ThermodynamicTemperature::new::<degree_celsius>(25.0);
        let to = ThermodynamicTemperature::new::<degree_fahrenheit>(77.0);
        let delta = temperature_difference(from, to);
        assert_relative_eq!(delta.get::<delta_celsius>(), 0.0, epsilon = 1e-12);
    }
}
