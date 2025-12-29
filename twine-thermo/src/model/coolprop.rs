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
    f64::{MolarMass, Pressure},
    mass_density::kilogram_per_cubic_meter,
    molar_mass::kilogram_per_mole,
    pressure::pascal,
    thermodynamic_temperature::kelvin,
};

use crate::{
    PropertyError, State,
    capability::{HasPressure, ThermoModel},
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

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        molar_mass::gram_per_mole,
        pressure::megapascal,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::fluid::CarbonDioxide;

    #[test]
    fn coolprop_co2_molar_mass_matches_expected() {
        let model = CoolProp::<CarbonDioxide>::new().unwrap();
        let molar_mass = model.molar_mass().unwrap();
        assert_relative_eq!(molar_mass.get::<gram_per_mole>(), 44.0098);
    }

    #[test]
    fn coolprop_co2_pressure_valid_state() {
        let model = CoolProp::<CarbonDioxide>::new().unwrap();
        let state = State::new(
            ThermodynamicTemperature::new::<degree_celsius>(42.0),
            MassDensity::new::<kilogram_per_cubic_meter>(670.0),
            CarbonDioxide,
        );
        let pressure = model.pressure(&state).unwrap();
        assert_relative_eq!(pressure.get::<megapascal>(), 11.3362, epsilon = 1e-4);
    }
}
