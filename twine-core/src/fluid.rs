//! Traits related to modeling fluid properties.
//!
//! This module provides traits for working with thermodynamic fluid states and properties.
//! The design enables flexible representations of fluid states while providing a
//! consistent interface for evaluating and modifying thermodynamic properties.

use std::fmt::Debug;

use thiserror::Error;
use uom::si::f64::{MassDensity, Pressure, SpecificHeatCapacity, ThermodynamicTemperature};

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

/// Trait for accessing temperature from a fluid state.
pub trait TemperatureProvider: FluidPropertyModel {
    /// Returns the temperature of the fluid state.
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature;
}

/// Trait for accessing density from a fluid state.
pub trait DensityProvider: FluidPropertyModel {
    /// Returns the density of the fluid state.
    fn density(&self, state: &Self::State) -> MassDensity;
}

/// Trait for accessing pressure from a fluid state.
pub trait PressureProvider: FluidPropertyModel {
    /// Returns the pressure of the fluid state.
    fn pressure(&self, state: &Self::State) -> Pressure;
}

/// Trait for accessing the specific heat at constant pressure (`cp`) of a fluid state.
///
/// # Errors
///
/// Returns an error if the specific heat is undefined at the given state, such
/// as within a two-phase region or near a critical point where `cp` may become
/// singular or undefined.
pub trait CpProvider: FluidPropertyModel {
    /// Returns the specific heat at constant pressure (`cp`) of the fluid state.
    fn cp(&self, state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError>;
}

/// Trait for accessing the specific heat at constant volume (`cv`) of a fluid state.
///
/// # Errors
///
/// Returns an error if the specific heat is undefined at the given state, such
/// as within a two-phase region or near a critical point where `cv` may become
/// singular or undefined.
pub trait CvProvider: FluidPropertyModel {
    /// Returns the specific heat at constant volume (`cv`) of the fluid state.
    fn cv(&self, state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError>;
}

/// Trait for creating a fluid state from temperature.
pub trait NewStateFromTemperature: FluidPropertyModel {
    /// Creates a new fluid state from the provided temperature.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the temperature is invalid or if the calculation fails.
    fn new_state_from_temperature(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from density.
pub trait NewStateFromDensity: FluidPropertyModel {
    /// Creates a new fluid state from the provided density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// density, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the density is invalid or if the calculation fails.
    fn new_state_from_density(
        &self,
        reference: &Self::State,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure.
pub trait NewStateFromPressure: FluidPropertyModel {
    /// Creates a new fluid state from the provided pressure.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the pressure is invalid or if the calculation fails.
    fn new_state_from_pressure(
        &self,
        reference: &Self::State,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from temperature and density.
pub trait NewStateFromTemperatureDensity: FluidPropertyModel {
    /// Creates a new fluid state from the provided temperature and density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature and density, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the temperature or density is invalid or if the calculation fails.
    fn new_state_from_temperature_density(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from temperature and pressure.
pub trait NewStateFromTemperaturePressure: FluidPropertyModel {
    /// Creates a new fluid state from the provided temperature and pressure.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// temperature and pressure, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the temperature or pressure is invalid or if the calculation fails.
    fn new_state_from_temperature_pressure(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Trait for creating a fluid state from pressure and density.
pub trait NewStateFromPressureDensity: FluidPropertyModel {
    /// Creates a new fluid state from the provided pressure and density.
    ///
    /// Uses the reference state to preserve other properties when possible.
    /// If the fluid model cannot preserve certain properties when changing
    /// pressure and density, it should document this behavior.
    ///
    /// # Errors
    ///
    /// Returns an error if the pressure or density is invalid or if the calculation fails.
    fn new_state_from_pressure_density(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Errors that occur when constructing a new fluid state from input properties.
///
/// `FluidStateError` represents failures encountered during the creation of a fluid
/// state from thermodynamic inputs such as temperature, pressure, or density.
/// This includes:
///
/// - Physically invalid or inconsistent input values.
/// - Numerical or internal calculation errors during state construction.
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
    /// physical validity — such as division by zero, convergence failure, or
    /// floating-point overflow.
    #[error("Calculation error: {0}")]
    CalculationError(String),
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
    #[error("Calculation error: {0}")]
    CalculationError(String),
}
