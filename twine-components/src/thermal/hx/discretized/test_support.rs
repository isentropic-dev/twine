use std::convert::Infallible;

use twine_thermo::{
    PropertyError, State,
    capability::{HasEnthalpy, HasPressure, StateFrom, ThermoModel},
    units::{SpecificEnthalpy, TemperatureDifference},
};
use uom::si::{
    f64::{
        MassDensity, Pressure, SpecificHeatCapacity, TemperatureInterval, ThermodynamicTemperature,
    },
    mass_density::kilogram_per_cubic_meter,
    pressure::pascal,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::kelvin,
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct TestFluid;

#[derive(Debug, Clone, Copy)]
pub(super) struct TestThermoModel {
    cp: SpecificHeatCapacity,
    pressure: Pressure,
    density: MassDensity,
}

impl TestThermoModel {
    pub(super) fn new() -> Self {
        Self {
            cp: SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1000.0),
            pressure: Pressure::new::<pascal>(101_325.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
        }
    }

    /// Constant specific heat capacity used by this test model.
    pub(super) fn cp(&self) -> SpecificHeatCapacity {
        self.cp
    }
}

impl ThermoModel for TestThermoModel {
    type Fluid = TestFluid;
}

impl HasPressure for TestThermoModel {
    fn pressure(&self, _state: &State<Self::Fluid>) -> Result<Pressure, PropertyError> {
        Ok(self.pressure)
    }
}

impl HasEnthalpy for TestThermoModel {
    fn enthalpy(&self, state: &State<Self::Fluid>) -> Result<SpecificEnthalpy, PropertyError> {
        let t_ref = ThermodynamicTemperature::new::<kelvin>(0.0);
        Ok(self.cp * state.temperature.minus(t_ref))
    }
}

impl StateFrom<(TestFluid, ThermodynamicTemperature, Pressure)> for TestThermoModel {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, temperature, _pressure): (TestFluid, ThermodynamicTemperature, Pressure),
    ) -> Result<State<TestFluid>, Self::Error> {
        Ok(State::new(temperature, self.density, fluid))
    }
}

impl StateFrom<(TestFluid, Pressure, SpecificEnthalpy)> for TestThermoModel {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, _pressure, enthalpy): (TestFluid, Pressure, SpecificEnthalpy),
    ) -> Result<State<TestFluid>, Self::Error> {
        let t_ref = ThermodynamicTemperature::new::<kelvin>(0.0);
        let delta_t: TemperatureInterval = enthalpy / self.cp;
        let temperature = t_ref + delta_t;
        Ok(State::new(temperature, self.density, fluid))
    }
}

pub(super) fn state(temp_kelvin: f64) -> State<TestFluid> {
    State::new(
        ThermodynamicTemperature::new::<kelvin>(temp_kelvin),
        MassDensity::new::<kilogram_per_cubic_meter>(1.0),
        TestFluid,
    )
}
