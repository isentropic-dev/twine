use uom::si::f64::{Pressure, SpecificHeatCapacity};

use crate::{
    PropertyError, State,
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy},
};

use super::ThermoModel;

pub trait HasPressure: ThermoModel {
    /// Returns the pressure for the given state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyError`] if the pressure cannot be calculated.
    fn pressure(&self, state: &State<Self::Fluid>) -> Result<Pressure, PropertyError>;
}

pub trait HasInternalEnergy: ThermoModel {
    /// Returns the specific internal energy for the given state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyError`] if the internal energy cannot be calculated.
    fn internal_energy(
        &self,
        state: &State<Self::Fluid>,
    ) -> Result<SpecificInternalEnergy, PropertyError>;
}

pub trait HasEnthalpy: ThermoModel {
    /// Returns the specific enthalpy for the given state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyError`] if the enthalpy cannot be calculated.
    fn enthalpy(&self, state: &State<Self::Fluid>) -> Result<SpecificEnthalpy, PropertyError>;
}

pub trait HasEntropy: ThermoModel {
    /// Returns the specific entropy for the given state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyError`] if the entropy cannot be calculated.
    fn entropy(&self, state: &State<Self::Fluid>) -> Result<SpecificEntropy, PropertyError>;
}

pub trait HasCp: ThermoModel {
    /// Returns the specific heat capacity at constant pressure for the given state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyError`] if `cp` cannot be calculated.
    fn cp(&self, state: &State<Self::Fluid>) -> Result<SpecificHeatCapacity, PropertyError>;
}

pub trait HasCv: ThermoModel {
    /// Returns the specific heat capacity at constant volume for the given state.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyError`] if `cv` cannot be calculated.
    fn cv(&self, state: &State<Self::Fluid>) -> Result<SpecificHeatCapacity, PropertyError>;
}
