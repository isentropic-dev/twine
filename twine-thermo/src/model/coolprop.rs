mod error;

use std::{
    marker::PhantomData,
    sync::{Mutex, MutexGuard},
};

use rfluids::{
    io::{FluidInputPair, FluidParam, FluidTrivialParam},
    native::AbstractState,
};
use uom::si::{
    available_energy::joule_per_kilogram,
    f64::{MassDensity, MolarMass, Pressure, SpecificHeatCapacity, ThermodynamicTemperature},
    mass_density::kilogram_per_cubic_meter,
    molar_mass::kilogram_per_mole,
    pressure::pascal,
    specific_heat_capacity::joule_per_kilogram_kelvin,
    thermodynamic_temperature::kelvin,
};

use crate::{
    PropertyError, State,
    capability::{
        HasCp, HasCv, HasEnthalpy, HasEntropy, HasInternalEnergy, HasPressure, StateFrom,
        ThermoModel,
    },
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy},
};

pub use error::CoolPropError;

/// Trait used to mark fluids as usable with the [`CoolProp`] model.
///
/// Implementors provide the backend and fluid identifiers needed to construct a
/// `CoolProp` `AbstractState`.
pub trait CoolPropFluid: Default + Send + Sync + 'static {
    const BACKEND: &'static str;
    const NAME: &'static str;
}

/// A fluid property model backed by `CoolProp`.
pub struct CoolProp<F: CoolPropFluid> {
    state: Mutex<AbstractState>,
    _f: PhantomData<F>,
}

impl<F: CoolPropFluid> ThermoModel for CoolProp<F> {
    type Fluid = F;
}

impl<F: CoolPropFluid> CoolProp<F> {
    /// Construct a new CoolProp-backed model instance.
    ///
    /// # Errors
    ///
    /// Returns [`CoolPropError`] if the underlying `AbstractState` cannot be
    /// created for the given `F::BACKEND` and `F::NAME`.
    pub fn new() -> Result<Self, CoolPropError> {
        let state = AbstractState::new(F::BACKEND, F::NAME)?;
        Ok(Self {
            state: Mutex::new(state),
            _f: PhantomData,
        })
    }

    /// Returns the molar mass of the fluid.
    ///
    /// # Errors
    ///
    /// Returns [`CoolPropError`] if the call fails.
    pub fn molar_mass(&self) -> Result<MolarMass, CoolPropError> {
        let abstract_state = self.state.lock()?;
        let molar_mass = abstract_state.keyed_output(FluidTrivialParam::MolarMass)?;
        Ok(MolarMass::new::<kilogram_per_mole>(molar_mass))
    }

    /// Locks the underlying `AbstractState` and updates it from `state`.
    fn lock_with_state(
        &self,
        state: &State<F>,
    ) -> Result<MutexGuard<'_, AbstractState>, CoolPropError> {
        let mut abstract_state = self.state.lock()?;
        abstract_state.update(
            FluidInputPair::DMassT,
            state.density.get::<kilogram_per_cubic_meter>(),
            state.temperature.get::<kelvin>(),
        )?;
        Ok(abstract_state)
    }
}

impl<F: CoolPropFluid> HasPressure for CoolProp<F> {
    fn pressure(&self, state: &State<Self::Fluid>) -> Result<Pressure, PropertyError> {
        let abstract_state = self.lock_with_state(state)?;
        let pressure = abstract_state
            .keyed_output(FluidParam::P)
            .map_err(CoolPropError::Rfluids)?;
        Ok(Pressure::new::<pascal>(pressure))
    }
}

impl<F: CoolPropFluid> HasInternalEnergy for CoolProp<F> {
    fn internal_energy(
        &self,
        state: &State<Self::Fluid>,
    ) -> Result<SpecificInternalEnergy, PropertyError> {
        let abstract_state = self.lock_with_state(state)?;
        let internal_energy = abstract_state
            .keyed_output(FluidParam::UMass)
            .map_err(CoolPropError::Rfluids)?;
        Ok(SpecificInternalEnergy::new::<joule_per_kilogram>(
            internal_energy,
        ))
    }
}

impl<F: CoolPropFluid> HasEnthalpy for CoolProp<F> {
    fn enthalpy(&self, state: &State<Self::Fluid>) -> Result<SpecificEnthalpy, PropertyError> {
        let abstract_state = self.lock_with_state(state)?;
        let enthalpy = abstract_state
            .keyed_output(FluidParam::HMass)
            .map_err(CoolPropError::Rfluids)?;
        Ok(SpecificEnthalpy::new::<joule_per_kilogram>(enthalpy))
    }
}

impl<F: CoolPropFluid> HasEntropy for CoolProp<F> {
    fn entropy(&self, state: &State<Self::Fluid>) -> Result<SpecificEntropy, PropertyError> {
        let abstract_state = self.lock_with_state(state)?;
        let entropy = abstract_state
            .keyed_output(FluidParam::SMass)
            .map_err(CoolPropError::Rfluids)?;
        Ok(SpecificEntropy::new::<joule_per_kilogram_kelvin>(entropy))
    }
}

impl<F: CoolPropFluid> HasCp for CoolProp<F> {
    fn cp(&self, state: &State<Self::Fluid>) -> Result<SpecificHeatCapacity, PropertyError> {
        let abstract_state = self.lock_with_state(state)?;
        let cp = abstract_state
            .keyed_output(FluidParam::CpMass)
            .map_err(CoolPropError::Rfluids)?;
        Ok(SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(cp))
    }
}

impl<F: CoolPropFluid> HasCv for CoolProp<F> {
    fn cv(&self, state: &State<Self::Fluid>) -> Result<SpecificHeatCapacity, PropertyError> {
        let abstract_state = self.lock_with_state(state)?;
        let cv = abstract_state
            .keyed_output(FluidParam::CvMass)
            .map_err(CoolPropError::Rfluids)?;
        Ok(SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(cv))
    }
}

impl<F: CoolPropFluid> StateFrom<(F, ThermodynamicTemperature, MassDensity)> for CoolProp<F> {
    type Error = CoolPropError;

    fn state_from(
        &self,
        (fluid, temperature, density): (F, ThermodynamicTemperature, MassDensity),
    ) -> Result<State<F>, Self::Error> {
        let mut abstract_state = self.state.lock()?;
        // Update CoolProp to validate the T-D state and surface invalid inputs early.
        abstract_state.update(
            FluidInputPair::DMassT,
            density.get::<kilogram_per_cubic_meter>(),
            temperature.get::<kelvin>(),
        )?;

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

impl<F: CoolPropFluid> StateFrom<(F, ThermodynamicTemperature, Pressure)> for CoolProp<F> {
    type Error = CoolPropError;

    fn state_from(
        &self,
        (fluid, temperature, pressure): (F, ThermodynamicTemperature, Pressure),
    ) -> Result<State<F>, Self::Error> {
        let mut abstract_state = self.state.lock()?;
        abstract_state.update(
            FluidInputPair::PT,
            pressure.get::<pascal>(),
            temperature.get::<kelvin>(),
        )?;

        let density = MassDensity::new::<kilogram_per_cubic_meter>(
            abstract_state.keyed_output(FluidParam::DMass)?,
        );

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        available_energy::kilojoule_per_kilogram,
        f64::{MassDensity, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        molar_mass::gram_per_mole,
        pressure::megapascal,
        specific_heat_capacity::{joule_per_kilogram_kelvin, kilojoule_per_kilogram_kelvin},
        thermodynamic_temperature::degree_celsius,
    };

    use crate::fluid::CarbonDioxide;

    fn co2_model() -> CoolProp<CarbonDioxide> {
        CoolProp::<CarbonDioxide>::new().unwrap()
    }

    fn co2_state() -> State<CarbonDioxide> {
        State::new(
            ThermodynamicTemperature::new::<degree_celsius>(42.0),
            MassDensity::new::<kilogram_per_cubic_meter>(670.0),
            CarbonDioxide,
        )
    }

    #[test]
    fn co2_molar_mass_matches_expected() {
        let model = co2_model();
        let molar_mass = model.molar_mass().unwrap();
        assert_relative_eq!(molar_mass.get::<gram_per_mole>(), 44.0098);
    }

    #[test]
    fn co2_pressure_matches_expected() {
        let model = co2_model();
        let state = co2_state();
        let pressure = model.pressure(&state).unwrap();
        assert_relative_eq!(pressure.get::<megapascal>(), 11.3362, epsilon = 1e-4);
    }

    #[test]
    fn co2_internal_energy_matches_expected() {
        let model = co2_model();
        let state = co2_state();
        let internal_energy = model.internal_energy(&state).unwrap();
        assert_relative_eq!(
            internal_energy.get::<kilojoule_per_kilogram>(),
            290.9565,
            epsilon = 1e-4
        );
    }

    #[test]
    fn co2_enthalpy_matches_expected() {
        let model = co2_model();
        let state = co2_state();
        let enthalpy = model.enthalpy(&state).unwrap();
        assert_relative_eq!(
            enthalpy.get::<kilojoule_per_kilogram>(),
            307.8761,
            epsilon = 1e-4
        );
    }

    #[test]
    fn co2_entropy_matches_expected() {
        let model = co2_model();
        let state = co2_state();
        let entropy = model.entropy(&state).unwrap();
        assert_relative_eq!(
            entropy.get::<kilojoule_per_kilogram_kelvin>(),
            1.3333,
            epsilon = 1e-4
        );
    }

    #[test]
    fn co2_cp_matches_expected() {
        let model = co2_model();
        let state = co2_state();
        let cp = model.cp(&state).unwrap();
        assert_relative_eq!(
            cp.get::<kilojoule_per_kilogram_kelvin>(),
            4.125,
            epsilon = 1e-4
        );
    }

    #[test]
    fn co2_cv_matches_expected() {
        let model = co2_model();
        let state = co2_state();
        let cv = model.cv(&state).unwrap();
        assert_relative_eq!(
            cv.get::<joule_per_kilogram_kelvin>(),
            980.5326,
            epsilon = 1e-4
        );
    }

    #[test]
    fn co2_state_from_temperature_density_roundtrips_via_temperature_pressure() {
        let model = co2_model();

        let temperature = ThermodynamicTemperature::new::<degree_celsius>(100.0);
        let density = MassDensity::new::<kilogram_per_cubic_meter>(10.0);

        let state = model.state_from((temperature, density)).unwrap();
        let pressure = model.pressure(&state).unwrap();
        let roundtrip = model.state_from((temperature, pressure)).unwrap();

        assert_relative_eq!(
            roundtrip.density.get::<kilogram_per_cubic_meter>(),
            density.get::<kilogram_per_cubic_meter>()
        );
    }
}
