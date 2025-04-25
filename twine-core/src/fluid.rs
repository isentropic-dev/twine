//! Traits for thermodynamic fluid modeling.
//!
//! This module provides traits for working with thermodynamic fluid states and properties.
//! The design allows for flexibility in how fluid states are represented, while providing
//! a consistent interface for accessing and modifying properties.

use std::fmt::Debug;
use uom::si::f64::{
    MassDensity,
    Pressure as UomPressure,
    ThermodynamicTemperature,
};
use uom::si::f64::Quantity;
use uom::si::energy_per_mass::joule_per_kilogram;
use uom::si::energy_per_mass::dimension::Dimension as EnergyPerMassDimension;

// Define SpecificEnthalpy as energy per mass (J/kg)
type SpecificEnthalpy = Quantity<EnergyPerMassDimension, uom::si::SI<f64>, f64>;

/// Base trait for fluid models.
///
/// This trait serves as the foundation for all fluid property traits.
/// Implementors define their own state representation through the associated
/// `State` type, which could be as simple as temperature and density for
/// ideal gases, or more complex structures for real fluids.
pub trait FluidModel: Sized + Clone + Debug {
    /// The type that represents the complete state of the fluid.
    type State: Clone + Debug;
}

/// Error type for fluid state operations.
#[derive(Debug, Clone)]
pub enum FluidStateError {
    /// The operation is not supported for this fluid model.
    Unsupported(String),
    /// The provided properties are inconsistent or invalid.
    InvalidProperties(String),
    /// A calculation error occurred.
    CalculationError(String),
}

impl std::fmt::Display for FluidStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(msg) => write!(f, "Unsupported operation: {}", msg),
            Self::InvalidProperties(msg) => write!(f, "Invalid properties: {}", msg),
            Self::CalculationError(msg) => write!(f, "Calculation error: {}", msg),
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
    fn pressure(&self, state: &Self::State) -> UomPressure;
}

/// Trait for accessing density from a fluid state.
pub trait DensityProvider: FluidModel {
    /// Returns the density of the fluid state.
    fn density(&self, state: &Self::State) -> MassDensity;
}

/// Trait for accessing enthalpy from a fluid state.
pub trait EnthalpyProvider: FluidModel {
    /// Returns the specific enthalpy of the fluid state.
    fn enthalpy(&self, state: &Self::State) -> SpecificEnthalpy;
}

/// Trait for creating a fluid state from temperature and pressure.
pub trait FromTemperaturePressure: FluidModel {
    /// Creates a new fluid state from temperature and pressure.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature and pressure, it should document this behavior.
    fn new_state_from_temperature_pressure(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature, 
        pressure: UomPressure
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure and enthalpy.
pub trait FromPressureEnthalpy: FluidModel {
    /// Creates a new fluid state from pressure and enthalpy.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure and enthalpy, it should document this behavior.
    fn new_state_from_pressure_enthalpy(
        &self,
        reference: &Self::State,
        pressure: UomPressure,
        enthalpy: SpecificEnthalpy
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from temperature.
pub trait FromTemperature: FluidModel {
    /// Creates a new fluid state from temperature.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature, it should document this behavior.
    fn new_state_from_temperature(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure.
pub trait FromPressure: FluidModel {
    /// Creates a new fluid state from pressure.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure, it should document this behavior.
    fn new_state_from_pressure(
        &self,
        reference: &Self::State,
        pressure: UomPressure
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from density.
pub trait FromDensity: FluidModel {
    /// Creates a new fluid state from density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// density, it should document this behavior.
    fn new_state_from_density(
        &self,
        reference: &Self::State,
        density: MassDensity
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure and density.
pub trait FromPressureDensity: FluidModel {
    /// Creates a new fluid state from pressure and density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure and density, it should document this behavior.
    fn new_state_from_pressure_density(
        &self,
        reference: &Self::State,
        pressure: UomPressure,
        density: MassDensity
    ) -> Result<Self::State, FluidStateError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::{
        mass_density::kilogram_per_cubic_meter,
        pressure::pascal,
        thermodynamic_temperature::kelvin,
        energy_per_mass::joule_per_kilogram,
    };
    // Define the universal gas constant (J/(mol·K))
    const UNIVERSAL_GAS_CONSTANT: f64 = 8.314;

    /// A state representation for an ideal gas.
    #[derive(Debug, Clone)]
    struct IdealGasState {
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    }

    /// A model for ideal gas calculations.
    #[derive(Debug, Clone)]
    struct IdealGasModel {
        molar_mass: f64, // kg/mol
        gas_constant: f64, // J/(mol·K)
        specific_heat_capacity: f64, // J/(kg·K)
    }

    impl IdealGasModel {
        fn new() -> Self {
            Self {
                molar_mass: 0.029, // Default to air (kg/mol)
                gas_constant: UNIVERSAL_GAS_CONSTANT,
                specific_heat_capacity: 1005.0, // J/(kg·K) for air
            }
        }

        // Calculate pressure using ideal gas law: P = ρRT/M
        fn calculate_pressure(&self, state: &IdealGasState) -> UomPressure {
            let rho = state.density.get::<kilogram_per_cubic_meter>();
            let temp = state.temperature.get::<kelvin>();
            let pressure_value = rho * self.gas_constant * temp / self.molar_mass;
            UomPressure::new::<pascal>(pressure_value)
        }

        // Calculate enthalpy: h = cp*T for ideal gas
        fn calculate_enthalpy(&self, state: &IdealGasState) -> SpecificEnthalpy {
            let temp = state.temperature.get::<kelvin>();
            let enthalpy_value = self.specific_heat_capacity * temp;
            SpecificEnthalpy::new::<joule_per_kilogram>(enthalpy_value)
        }

        // Calculate density from temperature and pressure: ρ = PM/(RT)
        fn calculate_density(&self, temperature: ThermodynamicTemperature, pressure: UomPressure) -> MassDensity {
            let temp = temperature.get::<kelvin>();
            let p = pressure.get::<pascal>();
            let density_value = p * self.molar_mass / (self.gas_constant * temp);
            MassDensity::new::<kilogram_per_cubic_meter>(density_value)
        }

        // Calculate temperature from pressure and density: T = PM/(ρR)
        fn calculate_temperature(&self, pressure: UomPressure, density: MassDensity) -> ThermodynamicTemperature {
            let p = pressure.get::<pascal>();
            let rho = density.get::<kilogram_per_cubic_meter>();
            let temp_value = p * self.molar_mass / (rho * self.gas_constant);
            ThermodynamicTemperature::new::<kelvin>(temp_value)
        }

        // Calculate temperature from enthalpy: T = h/cp for ideal gas
        fn calculate_temperature_from_enthalpy(&self, enthalpy: SpecificEnthalpy) -> ThermodynamicTemperature {
            let h = enthalpy.get::<joule_per_kilogram>();
            let temp_value = h / self.specific_heat_capacity;
            ThermodynamicTemperature::new::<kelvin>(temp_value)
        }
    }

    impl FluidModel for IdealGasModel {
        type State = IdealGasState;
    }
    
    // Helper method to create a new state (moved from the trait implementation)
    impl IdealGasModel {
        fn new_state(&self, temperature: ThermodynamicTemperature, density: MassDensity) -> IdealGasState {
            IdealGasState {
                temperature,
                density,
            }
        }
    }

    impl TemperatureProvider for IdealGasModel {
        fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature {
            state.temperature
        }
    }

    impl PressureProvider for IdealGasModel {
        fn pressure(&self, state: &Self::State) -> UomPressure {
            self.calculate_pressure(state)
        }
    }

    impl DensityProvider for IdealGasModel {
        fn density(&self, state: &Self::State) -> MassDensity {
            state.density
        }
    }
    
    impl EnthalpyProvider for IdealGasModel {
        fn enthalpy(&self, state: &Self::State) -> SpecificEnthalpy {
            self.calculate_enthalpy(state)
        }
    }
    
    impl FromTemperaturePressure for IdealGasModel {
        fn new_state_from_temperature_pressure(
            &self,
            _reference: &Self::State,
            temperature: ThermodynamicTemperature, 
            pressure: UomPressure
        ) -> Result<Self::State, FluidStateError> {
            let density = self.calculate_density(temperature, pressure);
            Ok(self.new_state(temperature, density))
        }
    }
    
    impl FromPressureEnthalpy for IdealGasModel {
        fn new_state_from_pressure_enthalpy(
            &self,
            _reference: &Self::State,
            pressure: UomPressure,
            enthalpy: SpecificEnthalpy
        ) -> Result<Self::State, FluidStateError> {
            // For ideal gas: h = cp*T, so T = h/cp
            let temperature = self.calculate_temperature_from_enthalpy(enthalpy);
            self.new_state_from_temperature_pressure(_reference, temperature, pressure)
        }
    }
    
    impl FromTemperature for IdealGasModel {
        fn new_state_from_temperature(
            &self,
            reference: &Self::State,
            temperature: ThermodynamicTemperature
        ) -> Result<Self::State, FluidStateError> {
            // For ideal gas, we'll assume constant pressure when changing temperature
            let pressure = self.pressure(reference);
            self.new_state_from_temperature_pressure(reference, temperature, pressure)
        }
    }
    
    impl FromPressure for IdealGasModel {
        fn new_state_from_pressure(
            &self,
            reference: &Self::State,
            pressure: UomPressure
        ) -> Result<Self::State, FluidStateError> {
            // For ideal gas, we'll assume constant temperature when changing pressure
            let temperature = self.temperature(reference);
            self.new_state_from_temperature_pressure(reference, temperature, pressure)
        }
    }
    
    impl FromDensity for IdealGasModel {
        fn new_state_from_density(
            &self,
            reference: &Self::State,
            density: MassDensity
        ) -> Result<Self::State, FluidStateError> {
            // For ideal gas, we'll assume constant temperature when changing density
            let temperature = self.temperature(reference);
            Ok(self.new_state(temperature, density))
        }
    }
    
    impl FromPressureDensity for IdealGasModel {
        fn new_state_from_pressure_density(
            &self,
            _reference: &Self::State,
            pressure: UomPressure,
            density: MassDensity
        ) -> Result<Self::State, FluidStateError> {
            let temperature = self.calculate_temperature(pressure, density);
            Ok(self.new_state(temperature, density))
        }
    }

    #[test]
    fn test_ideal_gas_properties() {
        // Create an ideal gas model
        let model = IdealGasModel::new();
        
        // Create an initial state for air at standard conditions
        let initial_state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.2),
        };
        
        // Use the initial state as a reference for further operations
        let state = initial_state.clone();

        // Test temperature getter
        assert_eq!(model.temperature(&state).get::<kelvin>(), 300.0);

        // Test pressure calculation
        let pressure = model.pressure(&state);
        println!("Pressure at 300K, 1.2 kg/m³: {} Pa", pressure.get::<pascal>());
        
        // Test creating state from temperature and pressure
        let tp_state = model.new_state_from_temperature_pressure(
            &state,
            ThermodynamicTemperature::new::<kelvin>(300.0),
            UomPressure::new::<pascal>(101325.0)
        ).unwrap();
        println!("Density at 300K, 101325 Pa: {} kg/m³", model.density(&tp_state).get::<kilogram_per_cubic_meter>());
        
        // Test creating state from pressure and enthalpy
        let ph_state = model.new_state_from_pressure_enthalpy(
            &state,
            UomPressure::new::<pascal>(101325.0),
            SpecificEnthalpy::new::<joule_per_kilogram>(300000.0)
        ).unwrap();
        println!("Temperature at 101325 Pa, 300000 J/kg: {} K", model.temperature(&ph_state).get::<kelvin>());
        
        // Test creating state from pressure and density
        let pd_state = model.new_state_from_pressure_density(
            &state,
            UomPressure::new::<pascal>(101325.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.2)
        ).unwrap();
        println!("Temperature at 101325 Pa, 1.2 kg/m³: {} K", model.temperature(&pd_state).get::<kelvin>());
        
        // Verify ideal gas law relationships
        let test_state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(273.15),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.293),
        };
        
        let test_pressure = model.pressure(&test_state);
        
        // P = ρRT/M
        let calculated_pressure = model.density(&test_state).get::<kilogram_per_cubic_meter>() * 
                                 model.gas_constant * 
                                 model.temperature(&test_state).get::<kelvin>() / 
                                 model.molar_mass;
        
        assert!((test_pressure.get::<pascal>() - calculated_pressure).abs() < 0.001);
    }
    
    #[test]
    fn test_new_traits() {
        // Create an ideal gas model
        let model = IdealGasModel::new();
        
        // Create an initial state
        let initial_state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.2),
        };
        
        // Test FromTemperature
        let temp_state = model.new_state_from_temperature(
            &initial_state,
            ThermodynamicTemperature::new::<kelvin>(350.0)
        ).unwrap();
        assert_eq!(model.temperature(&temp_state).get::<kelvin>(), 350.0);
        println!("New density after temperature change: {} kg/m³", 
                 model.density(&temp_state).get::<kilogram_per_cubic_meter>());
        
        // Test FromPressure
        let press_state = model.new_state_from_pressure(
            &initial_state,
            UomPressure::new::<pascal>(150000.0)
        ).unwrap();
        assert_eq!(model.pressure(&press_state).get::<pascal>(), 150000.0);
        println!("New density after pressure change: {} kg/m³", 
                 model.density(&press_state).get::<kilogram_per_cubic_meter>());
        
        // Test FromDensity
        let dens_state = model.new_state_from_density(
            &initial_state,
            MassDensity::new::<kilogram_per_cubic_meter>(1.5)
        ).unwrap();
        assert_eq!(model.density(&dens_state).get::<kilogram_per_cubic_meter>(), 1.5);
        println!("New pressure after density change: {} Pa", 
                 model.pressure(&dens_state).get::<pascal>());
    }
}
