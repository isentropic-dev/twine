use uom::{
    si::{
        f64::{TemperatureInterval, ThermodynamicTemperature},
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
        temperature_interval::{
            degree_celsius as delta_celsius, degree_rankine as delta_rankine,
            kelvin as delta_kelvin,
        },
        thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin},
    };

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
