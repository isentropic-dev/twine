//! Traits for thermodynamic fluid modeling.
//!
//! This module provides traits for working with thermodynamic fluid states and properties.
//! The design allows for flexibility in how fluid states are represented, while providing
//! a consistent interface for accessing and modifying properties.

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use uom::si::f64::{MassDensity, Pressure, ThermodynamicTemperature};

/// Base trait for fluid models.
///
/// This trait serves as the foundation for all fluid property traits.
/// Implementors define their own state representation through the associated
/// `State` type, which could be as simple as temperature and density for ideal
/// gases, or more complex structures for real fluids.
pub trait FluidModel: Sized + Clone + Debug {
    /// The type that represents the complete state of the fluid.
    type State: Clone + Debug;
}

/// Trait for accessing temperature from a fluid state.
pub trait TemperatureProvider: FluidModel {
    /// Returns the temperature of the fluid state.
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature;
}

/// Trait for accessing density from a fluid state.
pub trait DensityProvider: FluidModel {
    /// Returns the density of the fluid state.
    fn density(&self, state: &Self::State) -> MassDensity;
}

/// Trait for accessing pressure from a fluid state.
pub trait PressureProvider: FluidModel {
    /// Returns the pressure of the fluid state.
    fn pressure(&self, state: &Self::State) -> Pressure;
}

/// Trait for creating a fluid state from temperature.
pub trait FromTemperature: FluidModel {
    /// Creates a new fluid state from temperature.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the temperature is invalid or if the calculation fails.
    fn new_state_from_temperature(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from density.
pub trait FromDensity: FluidModel {
    /// Creates a new fluid state from density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// density, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the density is invalid or if the calculation fails.
    fn new_state_from_density(
        &self,
        reference: &Self::State,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure.
pub trait FromPressure: FluidModel {
    /// Creates a new fluid state from pressure.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the pressure is invalid or if the calculation fails.
    fn new_state_from_pressure(
        &self,
        reference: &Self::State,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from temperature and density.
pub trait FromTemperatureDensity: FluidModel {
    /// Creates a new fluid state from temperature and density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature and density, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the temperature or density is invalid or if the calculation fails.
    fn new_state_from_temperature_density(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from temperature and pressure.
pub trait FromTemperaturePressure: FluidModel {
    /// Creates a new fluid state from temperature and pressure.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature and pressure, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the temperature or pressure is invalid or if the calculation fails.
    fn new_state_from_temperature_pressure(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure and density.
pub trait FromPressureDensity: FluidModel {
    /// Creates a new fluid state from pressure and density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure and density, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the pressure or density is invalid or if the calculation fails.
    fn new_state_from_pressure_density(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Errors that can occur when creating a new fluid state.
#[derive(Debug, Clone)]
pub enum FluidStateError {
    /// The provided properties are inconsistent or invalid.
    InvalidProperties(String),
    /// A calculation error occurred.
    CalculationError(String),
}

impl Display for FluidStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProperties(msg) => write!(f, "Invalid properties: {msg}"),
            Self::CalculationError(msg) => write!(f, "Calculation error: {msg}"),
        }
    }
}

impl Error for FluidStateError {}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    use uom::si::{
        f64::SpecificHeatCapacity, mass_density::kilogram_per_cubic_meter, pressure::kilopascal,
        specific_heat_capacity::joule_per_kilogram_kelvin, thermodynamic_temperature::kelvin,
    };

    /// Defines the thermodynamic state of an ideal gas.
    #[derive(Debug, Clone)]
    struct IdealGasState {
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    }

    /// A model for an ideal gas fluid.
    #[derive(Debug, Clone)]
    struct IdealGasModel {
        specific_gas_constant: SpecificHeatCapacity,
    }

    impl IdealGasModel {
        /// Create a new ideal gas model for air.
        fn air() -> Self {
            Self {
                specific_gas_constant: SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(
                    287.053,
                ),
            }
        }

        // Calculate pressure using ideal gas law: P = ρRT.
        fn calculate_pressure(&self, state: &IdealGasState) -> Pressure {
            state.density * self.specific_gas_constant * state.temperature
        }

        // Calculate temperature from pressure and density: T = P/(ρR).
        fn calculate_temperature(
            &self,
            pressure: Pressure,
            density: MassDensity,
        ) -> ThermodynamicTemperature {
            // We're creating a `ThermodynamicTemperature` directly from a
            // calculated `TemperatureInterval`, which is safe because the
            // ideal gas law gives us an absolute temperature in Kelvin and
            // the `.value` accessor for a `TemperatureInterval` returns the
            // temperature difference in Kelvin.
            ThermodynamicTemperature::new::<kelvin>(
                (pressure / (density * self.specific_gas_constant)).value,
            )
        }

        // Calculate density from temperature and pressure: ρ = P/(RT).
        fn calculate_density(
            &self,
            temperature: ThermodynamicTemperature,
            pressure: Pressure,
        ) -> MassDensity {
            pressure / (self.specific_gas_constant * temperature)
        }
    }

    impl FluidModel for IdealGasModel {
        type State = IdealGasState;
    }

    impl TemperatureProvider for IdealGasModel {
        fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature {
            state.temperature
        }
    }

    impl DensityProvider for IdealGasModel {
        fn density(&self, state: &Self::State) -> MassDensity {
            state.density
        }
    }

    impl PressureProvider for IdealGasModel {
        fn pressure(&self, state: &Self::State) -> Pressure {
            self.calculate_pressure(state)
        }
    }

    impl FromTemperatureDensity for IdealGasModel {
        fn new_state_from_temperature_density(
            &self,
            _reference: &Self::State,
            temperature: ThermodynamicTemperature,
            density: MassDensity,
        ) -> Result<Self::State, FluidStateError> {
            Ok(IdealGasState {
                temperature,
                density,
            })
        }
    }

    impl FromTemperaturePressure for IdealGasModel {
        fn new_state_from_temperature_pressure(
            &self,
            _reference: &Self::State,
            temperature: ThermodynamicTemperature,
            pressure: Pressure,
        ) -> Result<Self::State, FluidStateError> {
            let density = self.calculate_density(temperature, pressure);
            Ok(IdealGasState {
                temperature,
                density,
            })
        }
    }

    impl FromPressureDensity for IdealGasModel {
        fn new_state_from_pressure_density(
            &self,
            _reference: &Self::State,
            pressure: Pressure,
            density: MassDensity,
        ) -> Result<Self::State, FluidStateError> {
            let temperature = self.calculate_temperature(pressure, density);
            Ok(IdealGasState {
                temperature,
                density,
            })
        }
    }

    impl FromTemperature for IdealGasModel {
        fn new_state_from_temperature(
            &self,
            reference: &Self::State,
            temperature: ThermodynamicTemperature,
        ) -> Result<Self::State, FluidStateError> {
            // Assume constant volume (density) when changing temperature.
            let density = self.density(reference);
            Ok(IdealGasState {
                temperature,
                density,
            })
        }
    }

    impl FromPressure for IdealGasModel {
        fn new_state_from_pressure(
            &self,
            reference: &Self::State,
            pressure: Pressure,
        ) -> Result<Self::State, FluidStateError> {
            // Assume constant volume (density) when changing pressure.
            let density = self.density(reference);
            let temperature = self.calculate_temperature(pressure, density);
            Ok(IdealGasState {
                temperature,
                density,
            })
        }
    }

    /// Air at standard sea-level conditions.
    fn air_at_sea_level() -> IdealGasState {
        IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(288.15),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.225),
        }
    }

    #[test]
    fn basic_properties() {
        let ideal_gas_air = IdealGasModel::air();
        let state = air_at_sea_level();

        assert_eq!(ideal_gas_air.temperature(&state).get::<kelvin>(), 288.15);
        assert_eq!(
            ideal_gas_air
                .density(&state)
                .get::<kilogram_per_cubic_meter>(),
            1.225
        );

        let pressure_in_kpa = ideal_gas_air.pressure(&state).get::<kilopascal>();
        assert!((pressure_in_kpa - 101.325).abs() < 1e-4);
    }

    #[test]
    fn temperature_pressure_relationships() {
        let ideal_gas_air = IdealGasModel::air();
        let initial_state = air_at_sea_level();

        // Increase the temperature at constant volume.
        let new_temperature = ThermodynamicTemperature::new::<kelvin>(350.0);
        let higher_temp_state = ideal_gas_air
            .new_state_from_temperature(&initial_state, new_temperature)
            .unwrap();

        // Verify the temperature changed while the density remained constant.
        assert_eq!(
            ideal_gas_air.temperature(&higher_temp_state),
            new_temperature
        );
        assert_eq!(
            ideal_gas_air.density(&higher_temp_state),
            ideal_gas_air.density(&initial_state)
        );

        // Verify pressure increased as expected based on the temperature ratio.
        let temp_ratio = new_temperature / ideal_gas_air.temperature(&initial_state);
        let expected_pressure = ideal_gas_air.pressure(&initial_state) * temp_ratio;
        assert_eq!(
            ideal_gas_air.pressure(&higher_temp_state),
            expected_pressure
        );

        // Next we double the pressure of the higher temperature state.
        let doubled_pressure = 2.0 * ideal_gas_air.pressure(&higher_temp_state);
        let doubled_pressure_state = ideal_gas_air
            .new_state_from_pressure(&higher_temp_state, doubled_pressure)
            .unwrap();

        // Verify the temperature also doubled.
        let expected_temperature_in_kelvin = 2.0 * new_temperature.get::<kelvin>();
        let actual_temperature_in_kelvin = ideal_gas_air
            .temperature(&doubled_pressure_state)
            .get::<kelvin>();
        assert_eq!(actual_temperature_in_kelvin, expected_temperature_in_kelvin);
    }
}
