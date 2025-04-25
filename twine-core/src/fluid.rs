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

/// Errors that can occur when creating fluid states.
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
        molar_mass: MolarMass,
        gas_constant: MolarHeatCapacity,
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

    /// Creates a standard air state at the given temperature
    fn create_standard_air_state(temperature: f64) -> (IdealGasModel, IdealGasState) {
        let model = IdealGasModel::air();
        let state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(temperature),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.2),
        };
        (model, state)
    }

    #[test]
    fn test_ideal_gas_basic_properties() {
        let (model, state) = create_standard_air_state(300.0);

        // Test temperature getter
        assert_eq!(model.temperature(&state).get::<kelvin>(), 300.0);

        // Test density getter
        assert_eq!(model.density(&state).get::<kilogram_per_cubic_meter>(), 1.2);

        // Test pressure calculation
        let expected_pressure = 1.2 * 8.314 * 300.0 / 0.02896;
        let actual_pressure = model.pressure(&state).get::<pascal>();
        assert!((actual_pressure - expected_pressure).abs() < 0.1);
    }

    #[test]
    fn test_ideal_gas_law_relationship() {
        let (model, state) = create_standard_air_state(273.15);

        // Calculate pressure using the model
        let pressure = model.pressure(&state);

        // Calculate pressure using the ideal gas law directly
        let calculated_pressure =
            model.density(&state) * model.gas_constant * model.temperature(&state)
                / model.molar_mass;

        // Verify the pressures match
        assert!((pressure - calculated_pressure).abs() < Pressure::new::<pascal>(0.001));
    }

    #[test]
    fn test_temperature_pressure_relationship() {
        let (model, initial_state) = create_standard_air_state(300.0);

        // Create a new state with a different temperature
        let new_temp = ThermodynamicTemperature::new::<kelvin>(350.0);
        let temp_state = model
            .new_state_from_temperature(&initial_state, new_temp)
            .unwrap();

        // Verify temperature changed
        assert_eq!(model.temperature(&temp_state).get::<kelvin>(), 350.0);

        // Verify density remained constant (constant volume)
        assert_eq!(
            model.density(&temp_state).get::<kilogram_per_cubic_meter>(),
            model
                .density(&initial_state)
                .get::<kilogram_per_cubic_meter>()
        );

        // Verify pressure increased with temperature (constant volume)
        assert!(
            model.pressure(&temp_state).get::<pascal>()
                > model.pressure(&initial_state).get::<pascal>()
        );

        // Calculate expected pressure ratio based on temperature ratio
        let expected_pressure_ratio = 350.0 / 300.0;
        let actual_pressure_ratio = model.pressure(&temp_state).get::<pascal>()
            / model.pressure(&initial_state).get::<pascal>();

        assert!((actual_pressure_ratio - expected_pressure_ratio).abs() < 0.001);
    }

    #[test]
    fn test_pressure_temperature_relationship() {
        let (model, initial_state) = create_standard_air_state(300.0);

        // Initial pressure
        let initial_pressure = model.pressure(&initial_state);

        // Double the pressure
        let new_pressure = initial_pressure * 2.0;
        let pressure_state = model
            .new_state_from_pressure(&initial_state, new_pressure)
            .unwrap();

        // Verify pressure changed
        assert!(
            (model.pressure(&pressure_state) - new_pressure).abs() < Pressure::new::<pascal>(0.001)
        );

        // Verify density remained constant (constant volume)
        assert_eq!(
            model
                .density(&pressure_state)
                .get::<kilogram_per_cubic_meter>(),
            model
                .density(&initial_state)
                .get::<kilogram_per_cubic_meter>()
        );

        // Verify temperature increased with pressure (constant volume)
        assert!(
            model.temperature(&pressure_state).get::<kelvin>()
                > model.temperature(&initial_state).get::<kelvin>()
        );

        // Calculate expected temperature ratio based on pressure ratio
        let expected_temp_ratio = 2.0; // Same as pressure ratio
        let actual_temp_ratio = model.temperature(&pressure_state).get::<kelvin>()
            / model.temperature(&initial_state).get::<kelvin>();

        assert!((actual_temp_ratio - expected_temp_ratio).abs() < 0.001);
    }

    #[test]
    fn test_temperature_pressure_density_relationships() {
        let (model, initial_state) = create_standard_air_state(300.0);

        // Create a new state with different temperature and pressure
        let new_temp = ThermodynamicTemperature::new::<kelvin>(350.0);
        let new_pressure = Pressure::new::<pascal>(101_325.0);

        let tp_state = model
            .new_state_from_temperature_pressure(&initial_state, new_temp, new_pressure)
            .unwrap();

        // Verify temperature and pressure match requested values
        assert_eq!(model.temperature(&tp_state).get::<kelvin>(), 350.0);
        assert_eq!(model.pressure(&tp_state).get::<pascal>(), 101_325.0);

        // Verify density is calculated correctly using the ideal gas law
        let expected_density = new_pressure * model.molar_mass / (model.gas_constant * new_temp);
        assert!(
            (model.density(&tp_state) - expected_density).abs()
                < MassDensity::new::<kilogram_per_cubic_meter>(0.001)
        );
    }

    #[test]
    fn test_pressure_density_temperature_relationships() {
        let (model, initial_state) = create_standard_air_state(300.0);

        // Create a new state with different pressure and density
        let new_pressure = Pressure::new::<pascal>(150_000.0);
        let new_density = MassDensity::new::<kilogram_per_cubic_meter>(1.5);

        let pd_state = model
            .new_state_from_pressure_density(&initial_state, new_pressure, new_density)
            .unwrap();

        // Verify pressure and density match requested values
        assert_eq!(model.pressure(&pd_state).get::<pascal>(), 150_000.0);
        assert_eq!(
            model.density(&pd_state).get::<kilogram_per_cubic_meter>(),
            1.5
        );

        // Verify temperature is calculated correctly using the ideal gas law
        let expected_temperature =
            new_pressure * model.molar_mass / (new_density * model.gas_constant);
        assert!(
            (model.temperature(&pd_state) - expected_temperature).abs()
                < ThermodynamicTemperature::new::<kelvin>(0.001)
        );
    }
}
