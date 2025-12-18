use std::{marker::PhantomData, sync::Mutex};

use rfluids::{io::FluidTrivialParam, native::AbstractState};
use thiserror::Error;
use uom::si::{f64::MolarMass, molar_mass::kilogram_per_mole};

/// Trait used to mark fluids as usable with the [`CoolProp`] model.
///
/// Implementors provide the backend and fluid identifiers needed to construct a
/// `CoolProp` `AbstractState`.
pub trait CoolPropFluid: Default + Send + Sync + 'static {
    const BACKEND: &'static str;
    const NAME: &'static str;
}

/// Errors returned by the [`CoolProp`] model.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoolPropError {
    #[error(transparent)]
    Rfluids(#[from] rfluids::native::CoolPropError),
    #[error("CoolProp state mutex poisoned")]
    Poisoned,
}

/// A fluid property model backed by `CoolProp`.
pub struct CoolProp<F: CoolPropFluid> {
    state: Mutex<AbstractState>,
    _f: PhantomData<F>,
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
        let state = self.state.lock().map_err(|_| CoolPropError::Poisoned)?;
        let molar_mass = state.keyed_output(FluidTrivialParam::MolarMass)?;
        Ok(MolarMass::new::<kilogram_per_mole>(molar_mass))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::molar_mass::gram_per_mole;

    use crate::fluid::CarbonDioxide;

    #[test]
    fn coolprop_co2_molar_mass_matches_expected() {
        let model = CoolProp::<CarbonDioxide>::new().unwrap();
        let molar_mass = model.molar_mass().unwrap();
        assert_relative_eq!(molar_mass.get::<gram_per_mole>(), 44.0098);
    }
}
