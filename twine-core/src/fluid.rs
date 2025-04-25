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
    SpecificEnthalpy,
};

/// Base trait for fluid models.
///
/// This trait serves as the foundation for all fluid property traits.
/// Implementors define their own state representation through the associated
/// `State` type, which could be as simple as temperature and density for
/// ideal gases, or more complex structures for real fluids.
pub trait FluidModel: Sized + Clone + Debug {
    /// The type that represents the complete state of the fluid.
    type State: Clone + Debug;
    
    /// Creates a new fluid state from temperature and density.
    ///
    /// This is the canonical way to create a fluid state, as temperature
    /// and density are sufficient to define the state for many fluids.
    fn new_state(&self, temperature: ThermodynamicTemperature, density: MassDensity) -> Self::State;
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
        specific_enthalpy::joule_per_kilogram,
    };
    // Define the universal gas constant (J/(mol·K))
    const UNIVERSAL_GAS_CONSTANT: f64 = 8.314;

    /// An ideal gas state that uses temperature and density as primary state variables
    /// and calculates other properties using the ideal gas law.
    #[derive(Debug, Clone)]
    struct IdealGasState {
        temperature: ThermodynamicTemperature,
        density: MassDensity,
        molar_mass: f64, // kg/mol
        gas_constant: f64, // J/(mol·K)
        specific_heat_capacity: f64, // J/(kg·K)
    }

    impl IdealGasState {
        // Calculate pressure using ideal gas law: P = ρRT/M
        fn calculate_pressure(&self) -> UomPressure {
            let rho = self.density.get::<kilogram_per_cubic_meter>();
            let temp = self.temperature.get::<kelvin>();
            let pressure_value = rho * self.gas_constant * temp / self.molar_mass;
            UomPressure::new::<pascal>(pressure_value)
        }

        // Calculate enthalpy: h = cp*T for ideal gas
        fn calculate_enthalpy(&self) -> SpecificEnthalpy {
            let temp = self.temperature.get::<kelvin>();
            let enthalpy_value = self.specific_heat_capacity * temp;
            SpecificEnthalpy::new::<joule_per_kilogram>(enthalpy_value)
        }
    }

    impl FluidState for IdealGasState {
        type State = Self;
        
        fn new(temperature: ThermodynamicTemperature, density: MassDensity) -> Self {
            Self {
                temperature,
                density,
                molar_mass: 0.029, // Default to air (kg/mol)
                gas_constant: UNIVERSAL_GAS_CONSTANT,
                specific_heat_capacity: 1005.0, // J/(kg·K) for air
            }
        }
    }

    impl TemperatureProvider for IdealGasState {
        fn temperature(&self) -> ThermodynamicTemperature {
            self.temperature
        }
    }

    impl PressureProvider for IdealGasState {
        fn pressure(&self) -> UomPressure {
            self.calculate_pressure()
        }
    }

    impl DensityProvider for IdealGasState {
        fn density(&self) -> MassDensity {
            self.density
        }
    }
    
    impl EnthalpyProvider for IdealGasState {
        fn enthalpy(&self) -> SpecificEnthalpy {
            self.calculate_enthalpy()
        }
    }
    
    impl FromTemperaturePressure for IdealGasState {
        fn new_from_temperature_pressure(
            temperature: ThermodynamicTemperature, 
            pressure: UomPressure
        ) -> Self {
            let temp = temperature.get::<kelvin>();
            let p = pressure.get::<pascal>();
            let density_value = p * 0.029 / (UNIVERSAL_GAS_CONSTANT * temp);
            let density = MassDensity::new::<kilogram_per_cubic_meter>(density_value);
            
            Self::new(temperature, density)
        }
    }
    
    impl FromPressureEnthalpy for IdealGasState {
        fn new_from_pressure_enthalpy(
            pressure: UomPressure,
            enthalpy: SpecificEnthalpy
        ) -> Self {
            // For ideal gas: h = cp*T, so T = h/cp
            let h = enthalpy.get::<joule_per_kilogram>();
            let cp = 1005.0; // J/(kg·K) for air
            let temp_value = h / cp;
            let temperature = ThermodynamicTemperature::new::<kelvin>(temp_value);
            
            Self::new_from_temperature_pressure(temperature, pressure)
        }
    }
    
    impl FromPressureDensity for IdealGasState {
        fn new_from_pressure_density(
            pressure: UomPressure,
            density: MassDensity
        ) -> Self {
            let p = pressure.get::<pascal>();
            let rho = density.get::<kilogram_per_cubic_meter>();
            let temp_value = p * 0.029 / (rho * UNIVERSAL_GAS_CONSTANT);
            let temperature = ThermodynamicTemperature::new::<kelvin>(temp_value);
            
            Self::new(temperature, density)
        }
    }

    #[test]
    fn test_ideal_gas_properties() {
        // Create an ideal gas state for air at standard conditions
        let state = IdealGasState::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.2)
        );

        // Test temperature getter
        assert_eq!(state.temperature().get::<kelvin>(), 300.0);

        // Test pressure calculation
        let pressure = state.pressure();
        println!("Pressure at 300K, 1.2 kg/m³: {} Pa", pressure.get::<pascal>());
        
        // Test creating state from temperature and pressure
        let tp_state = IdealGasState::new_from_temperature_pressure(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            UomPressure::new::<pascal>(101325.0)
        );
        println!("Density at 300K, 101325 Pa: {} kg/m³", tp_state.density().get::<kilogram_per_cubic_meter>());
        
        // Test creating state from pressure and enthalpy
        let ph_state = IdealGasState::new_from_pressure_enthalpy(
            UomPressure::new::<pascal>(101325.0),
            SpecificEnthalpy::new::<joule_per_kilogram>(300000.0)
        );
        println!("Temperature at 101325 Pa, 300000 J/kg: {} K", ph_state.temperature().get::<kelvin>());
        
        // Test creating state from pressure and density
        let pd_state = IdealGasState::new_from_pressure_density(
            UomPressure::new::<pascal>(101325.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.2)
        );
        println!("Temperature at 101325 Pa, 1.2 kg/m³: {} K", pd_state.temperature().get::<kelvin>());
        
        // Verify ideal gas law relationships
        let test_state = IdealGasState::new(
            ThermodynamicTemperature::new::<kelvin>(273.15),
            MassDensity::new::<kilogram_per_cubic_meter>(1.293)
        );
        
        let test_pressure = test_state.pressure();
        
        // P = ρRT/M
        let calculated_pressure = test_state.density.get::<kilogram_per_cubic_meter>() * 
                                 test_state.gas_constant * 
                                 test_state.temperature.get::<kelvin>() / 
                                 test_state.molar_mass;
        
        assert!((test_pressure.get::<pascal>() - calculated_pressure).abs() < 0.001);
    }
}
