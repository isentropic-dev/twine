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
pub trait FluidPropertyModel: Sized + Debug + Clone + PartialEq {
    /// Defines the complete thermodynamic state of the fluid.
    type State: Clone + Debug;
}

/// Capability to evaluate temperature.
pub trait ProvidesTemperature: FluidPropertyModel {
    fn temperature(&self, state: &Self::State) -> ThermodynamicTemperature;
}

/// Capability to evaluate density.
pub trait ProvidesDensity: FluidPropertyModel {
    fn density(&self, state: &Self::State) -> MassDensity;
}

/// Capability to evaluate pressure.
pub trait ProvidesPressure: FluidPropertyModel {
    fn pressure(&self, state: &Self::State) -> Pressure;
}

/// Capability to evaluate enthalpy.
pub trait ProvidesEnthalpy: FluidPropertyModel {
    fn enthalpy(&self, state: &Self::State) -> SpecificEnthalpy;
}

/// Capability to evaluate entropy.
pub trait ProvidesEntropy: FluidPropertyModel {
    fn entropy(&self, state: &Self::State) -> SpecificEntropy;
}

/// Capability to evaluate the specific heat at constant pressure (`cp`).
///
/// # Errors
///
/// Returns an error if the specific heat is undefined at the given state,
/// such as within a two-phase region or near a critical point.
pub trait ProvidesCp: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn cp(&self, state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError>;
}

/// Capability to evaluate the specific heat at constant volume (`cv`).
///
/// # Errors
///
/// Returns an error if the specific heat is undefined at the given state,
/// such as within a two-phase region or near a critical point.
pub trait ProvidesCv: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn cv(&self, state: &Self::State) -> Result<SpecificHeatCapacity, FluidPropertyError>;
}

/// Capability to produce a new fluid state with modified temperature.
///
/// The new state is produced by changing the temperature of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithTemperature: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_temperature(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified density.
///
/// The new state is produced by changing the density of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithDensity: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_density(
        &self,
        reference: &Self::State,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified pressure.
///
/// The new state is produced by changing the pressure of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithPressure: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_pressure(
        &self,
        reference: &Self::State,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified temperature and density.
///
/// The new state is produced by changing the temperature and density of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithTemperatureDensity: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_temperature_density(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified temperature and pressure.
///
/// The new state is produced by changing the temperature and pressure of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithTemperaturePressure: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_temperature_pressure(
        &self,
        reference: &Self::State,
        temperature: ThermodynamicTemperature,
        pressure: Pressure,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified pressure and density.
///
/// The new state is produced by changing the pressure and density of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithPressureDensity: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_pressure_density(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        density: MassDensity,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified pressure and enthalpy.
///
/// The new state is produced by changing the pressure and enthalpy of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithPressureEnthalpy: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_pressure_enthalpy(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        enthalpy: SpecificEnthalpy,
    ) -> Result<Self::State, FluidStateError>;
}

/// Capability to produce a new fluid state with modified pressure and entropy.
///
/// The new state is produced by changing the pressure and entropy of a reference state.
/// All other properties, such as composition or model-specific metadata,
/// may be preserved or recalculated depending on the model.
///
/// Implementations should document which parts of the reference state are retained,
/// recalculated, or affected by the update.
///
/// # Errors
///
/// Returns [`FluidStateError`] if the input is invalid or the operation fails.
pub trait WithPressureEntropy: FluidPropertyModel {
    #[allow(clippy::missing_errors_doc)]
    fn with_pressure_entropy(
        &self,
        reference: &Self::State,
        pressure: Pressure,
        entropy: SpecificEntropy,
    ) -> Result<Self::State, FluidStateError>;
}

/// Errors that occur when evaluating a fluid property.
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
    /// physical validity, such as division by zero or convergence failure.
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Errors that occur when modifying or constructing a fluid state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FluidStateError {
    /// One or more inputs are physically invalid or inconsistent.
    ///
    /// This error occurs when inputs violate physical laws, are out of the
    /// model's valid domain, or are otherwise unsuitable for creating a state.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// A numerical or internal calculation error occurred.
    ///
    /// This error occurs when the model fails due to issues unrelated to
    /// physical validity, such as division by zero or convergence failure.
    #[error("Calculation error: {0}")]
    CalculationError(String),
}
