//! Ideal gas equation of state helpers.
//!
//! These functions implement the ideal gas equation of state:
//! `p = ρ·R·T`.
//!
//! They are shared by models that assume ideal gas behavior (e.g. calorically perfect gases).

use uom::si::{
    f64::{MassDensity, Pressure, ThermodynamicTemperature},
    temperature_interval, thermodynamic_temperature,
};

use crate::units::SpecificGasConstant;

/// Computes pressure using the ideal gas equation of state.
#[must_use]
pub(crate) fn pressure(
    temperature: ThermodynamicTemperature,
    density: MassDensity,
    gas_constant: SpecificGasConstant,
) -> Pressure {
    density * gas_constant * temperature
}

/// Computes density using the ideal gas equation of state.
#[must_use]
pub(crate) fn density(
    temperature: ThermodynamicTemperature,
    pressure: Pressure,
    gas_constant: SpecificGasConstant,
) -> MassDensity {
    pressure / (gas_constant * temperature)
}

/// Computes temperature using the ideal gas equation of state.
///
/// Since [`SpecificGasConstant`] is associated with a `TemperatureInterval`,
/// the result must be manually converted to an absolute temperature.
#[must_use]
pub(crate) fn temperature(
    pressure: Pressure,
    density: MassDensity,
    gas_constant: SpecificGasConstant,
) -> ThermodynamicTemperature {
    let temperature = pressure / (density * gas_constant);
    ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(
        temperature.get::<temperature_interval::kelvin>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        mass_density::kilogram_per_cubic_meter, pressure::kilopascal,
        specific_heat_capacity::joule_per_kilogram_kelvin, thermodynamic_temperature::kelvin,
    };

    #[test]
    fn pressure_density_roundtrip() {
        let t = ThermodynamicTemperature::new::<kelvin>(350.0);
        let rho = MassDensity::new::<kilogram_per_cubic_meter>(1.25);
        let r = SpecificGasConstant::new::<joule_per_kilogram_kelvin>(287.053);

        let p = pressure(t, rho, r);
        let rho_2 = density(t, p, r);

        assert_relative_eq!(
            rho_2.get::<kilogram_per_cubic_meter>(),
            rho.get::<kilogram_per_cubic_meter>()
        );
    }

    #[test]
    fn pressure_temperature_roundtrip() {
        let p = Pressure::new::<kilopascal>(250.0);
        let rho = MassDensity::new::<kilogram_per_cubic_meter>(1.25);
        let r = SpecificGasConstant::new::<joule_per_kilogram_kelvin>(287.053);

        let t = temperature(p, rho, r);
        let p_2 = pressure(t, rho, r);

        assert_relative_eq!(p_2.get::<kilopascal>(), p.get::<kilopascal>());
    }

    #[test]
    fn temperature_density_roundtrip() {
        let t = ThermodynamicTemperature::new::<kelvin>(325.0);
        let p = Pressure::new::<kilopascal>(180.0);
        let r = SpecificGasConstant::new::<joule_per_kilogram_kelvin>(287.053);

        let rho = density(t, p, r);
        let t_2 = temperature(p, rho, r);

        assert_relative_eq!(t_2.get::<kelvin>(), t.get::<kelvin>());
    }
}
