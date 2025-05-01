use twine_core::thermo::{
    fluid::{
        CpProvider, CvProvider, DensityProvider, EnthalpyProvider, EntropyProvider,
        FluidPropertyError, FluidPropertyModel, FluidStateError, NewStateFromTemperature,
        TemperatureProvider,
    },
    units::{temperature_difference, SpecificEnthalpy, SpecificEntropy},
};
use uom::si::{
    f64::{MassDensity, SpecificHeatCapacity, ThermodynamicTemperature},
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::degree_celsius,
};

/// A fluid property model for incompressible liquids with constant cp and density.
///
/// Enthalpy and entropy are computed relative to a reference temperature, assuming:
/// - `h(T) = cp * (T - T_ref)`
/// - `s(T) = cp * ln(T / T_ref)`
#[derive(Debug, Clone)]
pub struct IncompressibleLiquid {
    pub density: MassDensity,
    pub cp: SpecificHeatCapacity,
    pub reference_temperature: ThermodynamicTemperature,
}

impl IncompressibleLiquid {
    /// Returns a preconfigured water model based on saturated liquid at 0 °C.
    #[must_use]
    pub fn water() -> Self {
        Self {
            density: MassDensity::new::<uom::si::mass_density::kilogram_per_cubic_meter>(998.2),
            cp: SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(4182.0),
            reference_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }
}

impl FluidPropertyModel for IncompressibleLiquid {
    type State = ThermodynamicTemperature;
}

impl TemperatureProvider for IncompressibleLiquid {
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature {
        *state
    }
}

impl DensityProvider for IncompressibleLiquid {
    fn density(&self, _state: &Self::State) -> MassDensity {
        self.density
    }
}

impl CpProvider for IncompressibleLiquid {
    fn cp(&self, _state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError> {
        Ok(self.cp)
    }
}

impl CvProvider for IncompressibleLiquid {
    fn cv(&self, _state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError> {
        Ok(self.cp) // cp = cv for incompressible fluids
    }
}

impl EnthalpyProvider for IncompressibleLiquid {
    fn enthalpy(&self, state: &Self::State) -> SpecificEnthalpy {
        self.cp * temperature_difference(self.reference_temperature, *state)
    }
}

impl EntropyProvider for IncompressibleLiquid {
    fn entropy(&self, state: &Self::State) -> SpecificEntropy {
        let t = state.value;
        let t_ref = self.reference_temperature.value;
        self.cp * (t / t_ref).ln()
    }
}

impl NewStateFromTemperature for IncompressibleLiquid {
    fn new_state_from_temperature(
        &self,
        _reference: &Self::State,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self::State, FluidStateError> {
        Ok(temperature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::{
        available_energy::joule_per_kilogram,
        mass_density::kilogram_per_cubic_meter,
        thermodynamic_temperature::{degree_celsius, kelvin},
    };

    #[test]
    fn basic_water_properties() {
        let water = IncompressibleLiquid::water();
        let state = ThermodynamicTemperature::new::<degree_celsius>(20.0);

        assert_relative_eq!(
            water.density(&state).get::<kilogram_per_cubic_meter>(),
            998.2
        );

        let ref_temp_in_k = water.reference_temperature.get::<kelvin>();
        let temp_in_k = water.temperature(&state).get::<kelvin>();
        let cp = water.cp(&state).unwrap().get::<joule_per_kilogram_kelvin>();

        assert_relative_eq!(temp_in_k, 293.15);
        assert_relative_eq!(
            water.enthalpy(&state).get::<joule_per_kilogram>(),
            // h(T) = cp * (T - T_ref)
            cp * (temp_in_k - ref_temp_in_k)
        );

        assert_relative_eq!(
            water.entropy(&state).get::<joule_per_kilogram_kelvin>(),
            // s(T) = cp * ln(T / T_ref)
            cp * (temp_in_k / ref_temp_in_k).ln()
        );
    }
}
