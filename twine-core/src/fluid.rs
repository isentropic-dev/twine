//! Traits for thermodynamic fluid modeling.
//!
//! This module provides traits for working with thermodynamic fluid states and properties.
//! The design allows for flexibility in how fluid states are represented, while providing
//! a consistent interface for accessing and modifying properties.

use std::fmt::Debug;
use uom::si::{
    mass_density::MassDensity,
    pressure::Pressure as UomPressure,
    thermodynamic_temperature::ThermodynamicTemperature,
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
    /// This method creates a new state rather than modifying the existing one,
    /// following Rust's preference for immutability where appropriate.
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

    /// A simple ideal gas state that only tracks temperature.
    #[derive(Debug, Clone)]
    struct IdealGasState {
        temperature: ThermodynamicTemperature,
    }

    impl FluidState for IdealGasState {
        type State = Self;
    }

    impl Temperature for IdealGasState {
        fn temperature(&self) -> ThermodynamicTemperature {
            self.temperature
        }

        fn with_temperature(&self, temperature: ThermodynamicTemperature) -> Self {
            Self { temperature }
        }
    }

    /// A more complex fluid state that tracks multiple properties.
    #[derive(Debug, Clone)]
    struct ComplexFluidState {
        temperature: ThermodynamicTemperature,
        pressure: UomPressure,
        density: MassDensity,
    }

    impl FluidState for ComplexFluidState {
        type State = Self;
    }

    impl Temperature for ComplexFluidState {
        fn temperature(&self) -> ThermodynamicTemperature {
            self.temperature
        }

        fn with_temperature(&self, temperature: ThermodynamicTemperature) -> Self {
            let mut new_state = self.clone();
            new_state.temperature = temperature;
            new_state
        }
    }

    impl Pressure for ComplexFluidState {
        fn pressure(&self) -> UomPressure {
            self.pressure
        }

        fn with_pressure(&self, pressure: UomPressure) -> Self {
            let mut new_state = self.clone();
            new_state.pressure = pressure;
            new_state
        }
    }

    impl Density for ComplexFluidState {
        fn density(&self) -> MassDensity {
            self.density
        }

        fn with_density(&self, density: MassDensity) -> Self {
            let mut new_state = self.clone();
            new_state.density = density;
            new_state
        }
    }

    #[test]
    fn test_ideal_gas_temperature() {
        let state = IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
        };

        assert_eq!(state.temperature().get::<kelvin>(), 300.0);

        let new_state = state.with_temperature(ThermodynamicTemperature::new::<kelvin>(350.0));
        assert_eq!(new_state.temperature().get::<kelvin>(), 350.0);
    }

    #[test]
    fn test_complex_fluid_properties() {
        let state = ComplexFluidState {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            pressure: UomPressure::new::<pascal>(101325.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
        };

        assert_eq!(state.temperature().get::<kelvin>(), 300.0);
        assert_eq!(state.pressure().get::<pascal>(), 101325.0);
        assert_eq!(state.density().get::<kilogram_per_cubic_meter>(), 1.0);

        let new_state = state
            .with_temperature(ThermodynamicTemperature::new::<kelvin>(350.0))
            .with_pressure(UomPressure::new::<pascal>(200000.0))
            .with_density(MassDensity::new::<kilogram_per_cubic_meter>(1.2));

        assert_eq!(new_state.temperature().get::<kelvin>(), 350.0);
        assert_eq!(new_state.pressure().get::<pascal>(), 200000.0);
        assert_eq!(new_state.density().get::<kilogram_per_cubic_meter>(), 1.2);
    }
}
