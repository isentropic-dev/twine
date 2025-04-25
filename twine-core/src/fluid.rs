//! Traits for thermodynamic fluid modeling.
//!
//! This module provides traits for working with thermodynamic fluid states and properties.
//! The design allows for flexibility in how fluid states are represented, while providing
//! a consistent interface for accessing and modifying properties.

use std::fmt::Debug;

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

/// Error type for fluid property calculations.
#[derive(Debug, Clone)]
pub enum FluidStateError {
    /// The provided properties are inconsistent or invalid.
    InvalidProperties(String),
    /// A calculation error occurred.
    CalculationError(String),
}

impl std::fmt::Display for FluidStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProperties(msg) => write!(f, "Invalid properties: {msg}"),
            Self::CalculationError(msg) => write!(f, "Calculation error: {msg}"),
        }
    }
}

impl std::error::Error for FluidStateError {}

/// Trait for accessing temperature from a fluid state.
pub trait TemperatureProvider: FluidModel {
    /// Returns the temperature of the fluid state.
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature;
}

/// Trait for accessing pressure from a fluid state.
pub trait PressureProvider: FluidModel {
    /// Returns the pressure of the fluid state.
    fn pressure(&self, state: &Self::State) -> Pressure;
}

/// Trait for accessing density from a fluid state.
pub trait DensityProvider: FluidModel {
    /// Returns the density of the fluid state.
    fn density(&self, state: &Self::State) -> MassDensity;
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

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    use uom::si::{
        f64::{MolarHeatCapacity, MolarMass},
        mass_density::kilogram_per_cubic_meter,
        molar_heat_capacity::joule_per_kelvin_mole,
        molar_mass::kilogram_per_mole,
        pressure::pascal,
        thermodynamic_temperature::kelvin,
    };

    /// A state representation for an ideal gas.
    #[derive(Debug, Clone)]
    struct IdealGasState {
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    }

    /// A model for ideal gas calculations.
    #[derive(Debug, Clone)]
    struct IdealGasModel {
        molar_mass: MolarMass,           // kg/mol
        gas_constant: MolarHeatCapacity, // J/(mol·K)
    }

    impl IdealGasModel {
        fn air() -> Self {
            Self {
                molar_mass: MolarMass::new::<kilogram_per_mole>(0.02896),
                gas_constant: MolarHeatCapacity::new::<joule_per_kelvin_mole>(8.314),
            }
        }

        // Calculate pressure using ideal gas law: P = ρRT/M
        fn calculate_pressure(&self, state: &IdealGasState) -> Pressure {
            // P = ρRT/M
            state.density * self.gas_constant * state.temperature / self.molar_mass
        }

        // Calculate temperature from pressure and density: T = PM/(ρR)
        fn calculate_temperature(
            &self,
            pressure: Pressure,
            density: MassDensity,
        ) -> ThermodynamicTemperature {
            // T = PM/(ρR)
            ThermodynamicTemperature::new::<kelvin>(
                (pressure * self.molar_mass / (density * self.gas_constant)).value,
            )
        }

        // Calculate density from temperature and pressure: ρ = PM/(RT)
        fn calculate_density(
            &self,
            temperature: ThermodynamicTemperature,
            pressure: Pressure,
        ) -> MassDensity {
            // ρ = PM/(RT)
            pressure * self.molar_mass / (self.gas_constant * temperature)
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
            reference: &Self::State,
            temperature: ThermodynamicTemperature,
            pressure: Pressure,
        ) -> Result<Self::State, FluidStateError> {
            let density = self.calculate_density(temperature, pressure);
            self.new_state_from_temperature_density(reference, temperature, density)
        }
    }

    impl FromPressureDensity for IdealGasModel {
        fn new_state_from_pressure_density(
            &self,
            reference: &Self::State,
            pressure: Pressure,
            density: MassDensity,
        ) -> Result<Self::State, FluidStateError> {
            let temperature = self.calculate_temperature(pressure, density);
            self.new_state_from_temperature_density(reference, temperature, density)
        }
    }

    impl FromTemperature for IdealGasModel {
        fn new_state_from_temperature(
            &self,
            reference: &Self::State,
            temperature: ThermodynamicTemperature,
        ) -> Result<Self::State, FluidStateError> {
            // Assume constant volume (density) when changing temperature
            let density = self.density(reference);
            self.new_state_from_temperature_density(reference, temperature, density)
        }
    }

    impl FromPressure for IdealGasModel {
        fn new_state_from_pressure(
            &self,
            reference: &Self::State,
            pressure: Pressure,
        ) -> Result<Self::State, FluidStateError> {
            // Assume constant volume (density) when changing pressure
            let temperature = self.calculate_temperature(pressure, self.density(reference));
            self.new_state_from_temperature_density(reference, temperature, self.density(reference))
        }
    }

    #[test]
    fn test_ideal_gas_properties() {
        let ideal_gas_air = IdealGasModel::air();

        // Create an initial state for air at standard conditions
        let state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.2),
        };

        // Test temperature getter
        assert_eq!(ideal_gas_air.temperature(&state).get::<kelvin>(), 300.0);

        // Test pressure calculation
        let pressure = ideal_gas_air.pressure(&state);
        println!(
            "Pressure at 300K, 1.2 kg/m³: {} Pa",
            pressure.get::<pascal>()
        );

        // Test creating state from temperature and pressure
        let tp_state = ideal_gas_air
            .new_state_from_temperature_pressure(
                &state,
                ThermodynamicTemperature::new::<kelvin>(300.0),
                Pressure::new::<pascal>(101_325.0),
            )
            .unwrap();
        println!(
            "Density at 300K, 101325 Pa: {} kg/m³",
            ideal_gas_air
                .density(&tp_state)
                .get::<kilogram_per_cubic_meter>()
        );

        // Verify ideal gas law relationships
        let test_state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(273.15),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.293),
        };

        let test_pressure = ideal_gas_air.pressure(&test_state);

        // P = ρRT/M - calculate directly with uom types
        let calculated_pressure = ideal_gas_air.density(&test_state)
            * ideal_gas_air.gas_constant
            * ideal_gas_air.temperature(&test_state)
            / ideal_gas_air.molar_mass;

        // Compare the pressures directly using uom's comparison
        assert!((test_pressure - calculated_pressure).abs() < Pressure::new::<pascal>(0.001));
    }

    #[test]
    fn test_new_traits() {
        // Create an ideal gas model
        let model = IdealGasModel::air();

        // Create an initial state
        let initial_state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.2),
        };

        // Test FromTemperature
        let temp_state = model
            .new_state_from_temperature(
                &initial_state,
                ThermodynamicTemperature::new::<kelvin>(350.0),
            )
            .unwrap();
        assert_eq!(model.temperature(&temp_state).get::<kelvin>(), 350.0);
        println!(
            "New density after temperature change: {} kg/m³",
            model.density(&temp_state).get::<kilogram_per_cubic_meter>()
        );

        // Test FromPressure
        let press_state = model
            .new_state_from_pressure(&initial_state, Pressure::new::<pascal>(150_000.0))
            .unwrap();
        assert_eq!(model.pressure(&press_state).get::<pascal>(), 150_000.0);
        println!(
            "New density after pressure change: {} kg/m³",
            model
                .density(&press_state)
                .get::<kilogram_per_cubic_meter>()
        );
    }
}
