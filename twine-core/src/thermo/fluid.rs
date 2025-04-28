//! Traits related to modeling fluid properties.
//!
//! This module provides traits for working with thermodynamic fluid states and properties.
//! The design enables flexible representations of fluid states while providing a
//! consistent interface for evaluating and modifying thermodynamic properties.

use std::fmt::Debug;

use thiserror::Error;
use uom::si::f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature};

use super::units::{SpecificEnthalpy, SpecificEntropy};

/// Base trait for fluid property models.
///
/// This trait serves as the foundation for all fluid property traits.
/// Implementors define their own state representation through the associated
/// `State` type, which could be as simple as temperature and density for ideal
/// gases, or more complex structures for real fluids.
pub trait FluidPropertyModel: Sized + Clone + Debug {
    /// Defines the complete thermodynamic state of the fluid.
    type State: Clone + Debug;
}

/// Provides access to the temperature of a fluid state.
pub trait TemperatureProvider: FluidPropertyModel {
    /// Returns the temperature of the fluid state.
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature;
}

/// Provides access to the density of a fluid state.
pub trait DensityProvider: FluidPropertyModel {
    /// Returns the density of the fluid state.
    fn density(&self, state: &Self::State) -> MassDensity;
}

/// Provides access to the pressure of a fluid state.
pub trait PressureProvider: FluidPropertyModel {
    /// Returns the pressure of the fluid state.
    fn pressure(&self, state: &Self::State) -> Pressure;
}

/// Provides access to the enthalpy of a fluid state.
pub trait EnthalpyProvider: FluidPropertyModel {
    /// Returns the enthalpy of the fluid state.
    fn enthalpy(&self, state: &Self::State) -> SpecificEnthalpy;
}

/// Provides access to the entropy of a fluid state.
pub trait EntropyProvider: FluidPropertyModel {
    /// Returns the entropy of the fluid state.
    fn entropy(&self, state: &Self::State) -> SpecificEntropy;
}

/// Provides access to the specific heat at constant pressure (`cp`) of a fluid state.
pub trait CpProvider: FluidPropertyModel {
    /// Returns the specific heat at constant pressure (`cp`) of the fluid state.
    ///
    /// # Errors
    ///
    /// Returns an error if the specific heat is undefined at the given state,
    /// such as within a two-phase region or near a critical point.
    fn cp(&self, state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError>;
}

/// Provides access to the specific heat at constant volume (`cv`) of a fluid state.
pub trait CvProvider: FluidPropertyModel {
    /// Returns the specific heat at constant volume (`cv`) of the fluid state.
    ///
    /// # Errors
    ///
    /// Returns an error if the specific heat is undefined at the given state,
    /// such as within a two-phase region or near a critical point.
    fn cv(&self, state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError>;
}

/// Creates a new fluid state from a provided temperature.
pub trait NewStateFromTemperature: FluidPropertyModel {
    /// Creates a new fluid state from the provided temperature.
    ///
    /// The new state is derived by modifying the temperature of the reference state.
    ///
    /// Preservation of other properties (such as density, pressure, phase
    /// information, or model-specific metadata) is determined by the fluid
    /// property model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided temperature is invalid or if the calculation fails.
    fn new_state_from_temperature(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided density.
pub trait NewStateFromDensity: FluidPropertyModel {
    /// Creates a new fluid state from the provided density.
    ///
    /// The new state is derived by modifying the density of the reference state.
    ///
    /// Preservation of other properties (such as temperature, pressure, phase
    /// information, or model-specific metadata) is determined by the fluid
    /// property model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided density is invalid or if the calculation fails.
    fn new_state_from_density(
        &self,
        reference: &Self::State,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided pressure.
pub trait NewStateFromPressure: FluidPropertyModel {
    /// Creates a new fluid state from the provided pressure.
    ///
    /// The new state is derived by modifying the pressure of the reference state.
    ///
    /// Preservation of other properties (such as temperature, density, phase
    /// information, or model-specific metadata) is determined by the fluid
    /// property model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided pressure is invalid or if the calculation fails.
    fn new_state_from_pressure(
        &self,
        reference: &Self::State,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided temperature and density.
pub trait NewStateFromTemperatureDensity: FluidPropertyModel {
    /// Creates a new fluid state from the provided temperature and density.
    ///
    /// The new state is derived by modifying the temperature and density of the
    /// reference state.
    ///
    /// Preservation of other properties is determined by the fluid property
    /// model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided temperature and density do not represent a valid
    /// thermodynamic state or if the calculation fails.
    fn new_state_from_temperature_density(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided temperature and pressure.
pub trait NewStateFromTemperaturePressure: FluidPropertyModel {
    /// Creates a new fluid state from the provided temperature and pressure.
    ///
    /// The new state is derived by modifying the temperature and pressure of
    /// the reference state.
    ///
    /// Preservation of other properties is determined by the fluid property
    /// model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided temperature and pressure do not represent a valid
    /// thermodynamic state or if the calculation fails.
    fn new_state_from_temperature_pressure(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided pressure and density.
pub trait NewStateFromPressureDensity: FluidPropertyModel {
    /// Creates a new fluid state from the provided pressure and density.
    ///
    /// The new state is derived by modifying the pressure and density of the
    /// reference state.
    ///
    /// Preservation of other properties is determined by the fluid property
    /// model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided pressure and density do not represent a valid
    /// thermodynamic state or if the calculation fails.
    fn new_state_from_pressure_density(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided pressure and enthalpy.
pub trait NewStateFromPressureEnthalpy: FluidPropertyModel {
    /// Creates a new fluid state from the provided pressure and enthalpy.
    ///
    /// The new state is derived by modifying the pressure and enthalpy of the
    /// reference state.
    ///
    /// Preservation of other properties is determined by the fluid property
    /// model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided pressure and enthalpy do not represent a valid
    /// thermodynamic state or if the calculation fails.
    fn new_state_from_pressure_enthalpy(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        enthalpy: SpecificEnthalpy,
    ) -> Result<Self::State, FluidStateError>;
}

/// Creates a new fluid state from a provided pressure and entropy.
pub trait NewStateFromPressureEntropy: FluidPropertyModel {
    /// Creates a new fluid state from the provided pressure and entropy.
    ///
    /// The new state is derived by modifying the pressure and entropy of the
    /// reference state.
    ///
    /// Preservation of other properties is determined by the fluid property
    /// model implementation.
    ///
    /// Implementations should document which aspects of the reference state are
    /// preserved or recalculated during state creation.
    ///
    /// # Errors
    ///
    /// Fails if the provided pressure and entropy do not represent a valid
    /// thermodynamic state or if the calculation fails.
    fn new_state_from_pressure_entropy(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        entropy: SpecificEntropy,
    ) -> Result<Self::State, FluidStateError>;
}

/// Errors that occur when evaluating fluid properties at a given state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FluidPropertyError {
    /// The requested property is undefined at the given state.
    ///
    /// This error occurs when attempting to evaluate a property that is not
    /// physically defined under current conditions — such as specific heat at
    /// constant pressure (`cp`) inside the two-phase region.
    #[error("Property '{property}' is undefined")]
    UndefinedProperty {
        /// Name of the property (e.g., "cp", "viscosity").
        property: &'static str,
        /// Optional explanation or context.
        context: Option<String>,
    },

    /// A numerical or internal calculation error occurred during property evaluation.
    ///
    /// This error occurs when the model fails due to issues unrelated to
    /// physical validity — such as division by zero or convergence failure.
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Errors that occur when constructing a new fluid state from input properties.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FluidStateError {
    /// One or more input values are physically invalid or inconsistent.
    ///
    /// This error occurs when inputs violate physical laws, are out of the
    /// model's valid domain, or are otherwise unsuitable for creating a state.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// A numerical or internal calculation error occurred during state construction.
    ///
    /// This error occurs when the model fails due to issues unrelated to
    /// physical validity — such as division by zero or convergence failure.
    #[error("Calculation error: {0}")]
    CalculationError(String),
}
