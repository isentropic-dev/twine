use twine_core::thermo::{
    fluid::{
        FluidPropertyError, FluidPropertyModel, FluidStateError, ProvidesCp, ProvidesCv,
        ProvidesDensity, ProvidesEnthalpy, ProvidesEntropy, ProvidesTemperature, WithTemperature,
    },
    units::{SpecificEnthalpy, SpecificEntropy, TemperatureDifference},
};
use uom::si::{
    f64::{MassDensity, SpecificHeatCapacity, ThermodynamicTemperature},
    mass_density::kilogram_per_cubic_meter,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::{degree_celsius, kelvin},
};

/// A fluid property model for incompressible liquids.
///
/// This model makes the following assumptions:
/// - Constant density (no compressibility effects)
/// - Constant specific heat capacity, used for both `cp` and `cv`
/// - No pressure dependence in property evaluations
/// - Single-phase, non-reactive behavior
///
/// Thermodynamic properties are computed relative to a reference temperature
/// `T_ref`, at which both specific enthalpy and entropy are defined to be zero:
///
/// ```text
/// h(T) = cp · (T - T_ref)
/// s(T) = cp · ln(T / T_ref)
/// ```
///
/// This model is well-suited for scenarios where computational efficiency is
/// prioritized over real-fluid fidelity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncompressibleLiquid {
    pub density: MassDensity,
    pub cp: SpecificHeatCapacity,
    pub reference_temperature: ThermodynamicTemperature,
}

impl IncompressibleLiquid {
    /// Returns a model for liquid water based on IAPWS reference conditions.
    ///
    /// - Density: 998.2 kg/m³
    /// - Specific heat capacity: 4,182 J/kg·K
    /// - Reference temperature: 0°C
    ///
    /// Suitable for water near atmospheric pressure in the temperature range of
    /// approximately 1°C to 100°C.
    ///
    /// Not appropriate for two-phase, freezing, or high-pressure conditions.
    #[must_use]
    pub fn water() -> Self {
        Self {
            density: MassDensity::new::<kilogram_per_cubic_meter>(998.2),
            cp: SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(4182.0),
            reference_temperature: ThermodynamicTemperature::new::<degree_celsius>(0.0),
        }
    }

    /// Returns a model for ethylene glycol using typical liquid properties.
    ///
    /// - Density: 1,113 kg/m³
    /// - Specific heat capacity: 2,380 J/kg·K
    /// - Reference temperature: 25°C
    ///
    /// Suitable for single-phase ethylene glycol near atmospheric pressure in
    /// the temperature range of approximately 10°C to 80°C.
    ///
    /// Not appropriate for two-phase, freezing, or high-pressure conditions.
    #[must_use]
    pub fn ethylene_glycol() -> Self {
        Self {
            density: MassDensity::new::<kilogram_per_cubic_meter>(1113.0),
            cp: SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(2380.0),
            reference_temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
        }
    }

    /// Returns a model for propylene glycol using typical liquid properties.
    ///
    /// - Density: 1,036 kg/m³
    /// - Specific heat capacity: 2,400 J/kg·K
    /// - Reference temperature: 25°C
    ///
    /// Suitable for single-phase propylene glycol near atmospheric pressure
    /// in the temperature range of approximately 10°C to 80°C.
    ///
    /// Not appropriate for two-phase, freezing, or high-pressure conditions.
    #[must_use]
    pub fn propylene_glycol() -> Self {
        Self {
            density: MassDensity::new::<kilogram_per_cubic_meter>(1036.0),
            cp: SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(2400.0),
            reference_temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
        }
    }
}

impl FluidPropertyModel for IncompressibleLiquid {
    type State = ThermodynamicTemperature;
}

impl ProvidesTemperature for IncompressibleLiquid {
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature {
        *state
    }
}

impl ProvidesDensity for IncompressibleLiquid {
    fn density(&self, _state: &Self::State) -> MassDensity {
        self.density
    }
}

impl ProvidesCp for IncompressibleLiquid {
    fn cp(&self, _state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError> {
        Ok(self.cp)
    }
}

impl ProvidesCv for IncompressibleLiquid {
    fn cv(&self, _state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError> {
        Ok(self.cp)
    }
}

impl ProvidesEnthalpy for IncompressibleLiquid {
    fn enthalpy(&self, state: &Self::State) -> SpecificEnthalpy {
        let delta_t = state.minus(self.reference_temperature);
        self.cp * delta_t
    }
}

impl ProvidesEntropy for IncompressibleLiquid {
    fn entropy(&self, state: &Self::State) -> SpecificEntropy {
        let t = state.get::<kelvin>();
        let t_ref = self.reference_temperature.get::<kelvin>();
        self.cp * (t / t_ref).ln()
    }
}

impl WithTemperature for IncompressibleLiquid {
    fn with_temperature(
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
        available_energy::kilojoule_per_kilogram,
        mass_density::kilogram_per_cubic_meter,
        thermodynamic_temperature::{degree_celsius, kelvin},
    };

    #[test]
    fn basic_water_properties() {
        let water = IncompressibleLiquid::water();
        let state = ThermodynamicTemperature::new::<degree_celsius>(20.0);

        assert_relative_eq!(water.temperature(&state).get::<kelvin>(), 293.15);

        assert_relative_eq!(
            water.density(&state).get::<kilogram_per_cubic_meter>(),
            998.2
        );

        assert_relative_eq!(
            water.enthalpy(&state).get::<kilojoule_per_kilogram>(),
            83.640,
        );

        assert_relative_eq!(
            water.entropy(&state).get::<joule_per_kilogram_kelvin>(),
            295.514,
            epsilon = 1e-4
        );
    }
}
