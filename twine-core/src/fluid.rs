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

/// Base trait for fluid state representations.
///
/// This trait serves as the foundation for all fluid property traits.
/// Implementors define their own state representation through the associated
/// `State` type, which could be as simple as a single temperature value for
/// ideal gases, or more complex structures for real fluids.
pub trait FluidState: Sized + Clone + Debug {
    /// The type that represents the complete state of the fluid.
    type State: Clone + Debug;
}

/// Trait for fluids with temperature as a property.
///
/// Implementors must provide methods to get and set the temperature
/// of a fluid state.
pub trait Temperature: FluidState {
    /// Returns the temperature of the fluid state.
    fn temperature(&self) -> ThermodynamicTemperature;

    /// Returns a new state with the specified temperature.
    ///
    /// The returned state has all other properties the same as the current state,
    /// with only the temperature changed to the provided value.
    fn with_temperature(&self, temperature: ThermodynamicTemperature) -> Self;
}

/// Trait for fluids with pressure as a property.
///
/// Implementors must provide methods to get and set the pressure
/// of a fluid state.
pub trait Pressure: FluidState {
    /// Returns the pressure of the fluid state.
    fn pressure(&self) -> UomPressure;

    /// Returns a new state with the specified pressure.
    ///
    /// The returned state has all other properties the same as the current state,
    /// with only the pressure changed to the provided value.
    fn with_pressure(&self, pressure: UomPressure) -> Self;
}

/// Trait for fluids with density as a property.
///
/// Implementors must provide methods to get and set the density
/// of a fluid state.
pub trait Density: FluidState {
    /// Returns the density of the fluid state.
    fn density(&self) -> MassDensity;

    /// Returns a new state with the specified density.
    ///
    /// The returned state has all other properties the same as the current state,
    /// with only the density changed to the provided value.
    fn with_density(&self, density: MassDensity) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::{
        mass_density::kilogram_per_cubic_meter,
        pressure::pascal,
        thermodynamic_temperature::kelvin,
    };
    use std::f64::consts::R as UNIVERSAL_GAS_CONSTANT;

    /// An ideal gas state that uses temperature as the primary state variable
    /// and calculates pressure and density using the ideal gas law.
    #[derive(Debug, Clone)]
    struct IdealGasState {
        temperature: ThermodynamicTemperature,
        molar_mass: f64, // kg/mol
        gas_constant: f64, // J/(mol·K)
    }

    impl IdealGasState {
        fn new(temperature: ThermodynamicTemperature, molar_mass: f64) -> Self {
            Self {
                temperature,
                molar_mass,
                gas_constant: 8.314, // J/(mol·K)
            }
        }

        // Calculate pressure using ideal gas law: P = ρRT/M
        fn calculate_pressure(&self, density: MassDensity) -> UomPressure {
            let rho = density.get::<kilogram_per_cubic_meter>();
            let temp = self.temperature.get::<kelvin>();
            let pressure_value = rho * self.gas_constant * temp / self.molar_mass;
            UomPressure::new::<pascal>(pressure_value)
        }

        // Calculate density using ideal gas law: ρ = PM/(RT)
        fn calculate_density(&self, pressure: UomPressure) -> MassDensity {
            let p = pressure.get::<pascal>();
            let temp = self.temperature.get::<kelvin>();
            let density_value = p * self.molar_mass / (self.gas_constant * temp);
            MassDensity::new::<kilogram_per_cubic_meter>(density_value)
        }

        // Calculate temperature using ideal gas law: T = PM/(ρR)
        fn calculate_temperature(&self, pressure: UomPressure, density: MassDensity) -> ThermodynamicTemperature {
            let p = pressure.get::<pascal>();
            let rho = density.get::<kilogram_per_cubic_meter>();
            let temp_value = p * self.molar_mass / (rho * self.gas_constant);
            ThermodynamicTemperature::new::<kelvin>(temp_value)
        }
    }

    impl FluidState for IdealGasState {
        type State = Self;
    }

    impl Temperature for IdealGasState {
        fn temperature(&self) -> ThermodynamicTemperature {
            self.temperature
        }

        fn with_temperature(&self, temperature: ThermodynamicTemperature) -> Self {
            let mut new_state = self.clone();
            new_state.temperature = temperature;
            new_state
        }
    }

    impl Pressure for IdealGasState {
        fn pressure(&self) -> UomPressure {
            // For ideal gas, we need density to calculate pressure
            // We'll use a standard air density at the current temperature
            let standard_density = self.calculate_density(UomPressure::new::<pascal>(101325.0));
            self.calculate_pressure(standard_density)
        }

        fn with_pressure(&self, pressure: UomPressure) -> Self {
            // For an ideal gas, changing pressure at constant volume means changing temperature
            let standard_density = self.calculate_density(UomPressure::new::<pascal>(101325.0));
            let new_temperature = self.calculate_temperature(pressure, standard_density);
            self.with_temperature(new_temperature)
        }
    }

    impl Density for IdealGasState {
        fn density(&self) -> MassDensity {
            // For ideal gas, we need pressure to calculate density
            // We'll use standard atmospheric pressure
            self.calculate_density(UomPressure::new::<pascal>(101325.0))
        }

        fn with_density(&self, density: MassDensity) -> Self {
            // For an ideal gas, changing density at constant pressure means changing temperature
            let standard_pressure = UomPressure::new::<pascal>(101325.0);
            let new_temperature = self.calculate_temperature(standard_pressure, density);
            self.with_temperature(new_temperature)
        }
    }

    #[test]
    fn test_ideal_gas_properties() {
        // Create an ideal gas state for air (molar mass ~0.029 kg/mol)
        let state = IdealGasState::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            0.029
        );

        // Test temperature getter and setter
        assert_eq!(state.temperature().get::<kelvin>(), 300.0);
        let new_state = state.with_temperature(ThermodynamicTemperature::new::<kelvin>(350.0));
        assert_eq!(new_state.temperature().get::<kelvin>(), 350.0);

        // Test pressure calculation and setter
        let pressure = state.pressure();
        println!("Pressure at 300K: {} Pa", pressure.get::<pascal>());
        
        let pressure_state = state.with_pressure(UomPressure::new::<pascal>(200000.0));
        println!("Temperature for 200kPa: {} K", pressure_state.temperature().get::<kelvin>());
        
        // Test density calculation and setter
        let density = state.density();
        println!("Density at 300K: {} kg/m³", density.get::<kilogram_per_cubic_meter>());
        
        let density_state = state.with_density(MassDensity::new::<kilogram_per_cubic_meter>(1.5));
        println!("Temperature for 1.5 kg/m³: {} K", density_state.temperature().get::<kelvin>());

        // Verify ideal gas law relationships
        let test_temp = ThermodynamicTemperature::new::<kelvin>(273.15);
        let test_state = state.with_temperature(test_temp);
        let test_pressure = test_state.pressure();
        let test_density = test_state.density();
        
        // P = ρRT/M
        let calculated_pressure = test_density.get::<kilogram_per_cubic_meter>() * 
                                 state.gas_constant * 
                                 test_temp.get::<kelvin>() / 
                                 state.molar_mass;
        
        assert!((test_pressure.get::<pascal>() - calculated_pressure).abs() < 0.001);
    }
}
