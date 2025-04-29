use twine_core::thermo::{
    fluid::{
        DensityProvider, FluidPropertyModel, FluidStateError, NewStateFromPressure,
        NewStateFromPressureDensity, NewStateFromTemperature, NewStateFromTemperatureDensity,
        NewStateFromTemperaturePressure, PressureProvider, TemperatureProvider,
    },
    units::SpecificGasConstant,
};
use uom::si::{
    f64::{MassDensity, Pressure, ThermodynamicTemperature},
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::kelvin,
};

/// A fluid property model using ideal gas assumptions.
///
/// `IdealGasModel` implements the ideal gas law, relating pressure, density,
/// and temperature by the equation `P = ρ⋅R⋅T`, where:
/// - `P` is pressure,
/// - `ρ` is mass density,
/// - `R` is the specific gas constant,
/// - `T` is absolute temperature.
///
/// The model assumes idealized behavior and is valid across the range of
/// pressures and temperatures where real gas effects are negligible.
///
/// # State Preservation
///
/// When creating a new fluid state by modifying a single property such as
/// temperature or pressure, the model assumes constant volume (i.e., constant
/// density) unless otherwise specified.
///
/// Specifically:
/// - Modifying temperature preserves the reference density, adjusting pressure accordingly.
/// - Modifying pressure preserves the reference density, adjusting temperature accordingly.
#[derive(Debug, Clone)]
pub struct IdealGasModel {
    specific_gas_constant: SpecificGasConstant,
}

/// Represents the thermodynamic state of an ideal gas.
///
/// `IdealGasState` holds the fundamental properties needed to characterize an
/// ideal gas: absolute temperature and mass density.
/// Pressure and other derived quantities can be computed dynamically from these
/// values using the ideal gas law.
#[derive(Debug, Clone)]
pub struct IdealGasState {
    pub temperature: ThermodynamicTemperature,
    pub density: MassDensity,
}

impl IdealGasModel {
    /// Creates a new ideal gas model with a user-specified specific gas constant.
    #[must_use]
    pub fn new(specific_gas_constant: SpecificGasConstant) -> Self {
        Self {
            specific_gas_constant,
        }
    }

    /// Returns a preconfigured ideal gas model for dry air.
    ///
    /// Assumes a specific gas constant of 287.053 J/(kg·K).
    #[must_use]
    pub fn dry_air() -> Self {
        Self::new(SpecificGasConstant::new::<joule_per_kilogram_kelvin>(
            287.053,
        ))
    }

    // Calculate pressure using ideal gas law: P = ρ⋅R⋅T.
    fn calculate_pressure(&self, state: &IdealGasState) -> Pressure {
        state.density * self.specific_gas_constant * state.temperature
    }

    // Calculate temperature from pressure and density: T = P/(ρ⋅R).
    fn calculate_temperature(
        &self,
        pressure: Pressure,
        density: MassDensity,
    ) -> ThermodynamicTemperature {
        // We're creating a `ThermodynamicTemperature` directly from a
        // calculated `TemperatureInterval`.
        // This is safe because the ideal gas law gives us an absolute
        // temperature in Kelvin, and the `.value` accessor for a
        // `TemperatureInterval` returns the temperature difference in Kelvin.
        ThermodynamicTemperature::new::<kelvin>(
            (pressure / (density * self.specific_gas_constant)).value,
        )
    }

    // Calculate density from temperature and pressure: ρ = P/(R⋅T).
    fn calculate_density(
        &self,
        temperature: ThermodynamicTemperature,
        pressure: Pressure,
    ) -> MassDensity {
        pressure / (self.specific_gas_constant * temperature)
    }
}

impl FluidPropertyModel for IdealGasModel {
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

impl NewStateFromTemperatureDensity for IdealGasModel {
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

impl NewStateFromTemperaturePressure for IdealGasModel {
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

impl NewStateFromPressureDensity for IdealGasModel {
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

impl NewStateFromTemperature for IdealGasModel {
    fn new_state_from_temperature(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self::State, FluidStateError> {
        // Assume constant volume (i.e., density) when changing temperature.
        let density = self.density(reference);
        Ok(IdealGasState {
            temperature,
            density,
        })
    }
}

impl NewStateFromPressure for IdealGasModel {
    fn new_state_from_pressure(
        &self,
        reference: &Self::State,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError> {
        // Assume constant volume (i.e., density) when changing pressure.
        let density = self.density(reference);
        let temperature = self.calculate_temperature(pressure, density);
        Ok(IdealGasState {
            temperature,
            density,
        })
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    use uom::si::{mass_density::kilogram_per_cubic_meter, pressure::kilopascal};

    /// Air at standard sea-level conditions.
    fn air_at_sea_level() -> IdealGasState {
        IdealGasState {
            temperature: ThermodynamicTemperature::new::<kelvin>(288.15),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.225),
        }
    }

    #[test]
    fn basic_properties() {
        let ideal_gas_air = IdealGasModel::dry_air();
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
        let ideal_gas_air = IdealGasModel::dry_air();
        let initial_state = air_at_sea_level();

        // Increase the temperature at constant density (i.e., volume).
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
