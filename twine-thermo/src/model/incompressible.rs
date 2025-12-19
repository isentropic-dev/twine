//! Incompressible liquid model with constant heat capacity.
//!
//! `Incompressible` implements a simple and widely-used engineering approximation for liquids:
//! constant density with a constant specific heat capacity.
//!
//! # Assumptions
//!
//! - Density is treated as constant (`ρ = ρ_ref`)
//! - Calorically perfect liquid: `cp` is constant (and `cv` is treated as equal to `cp`)
//! - Pressure effects are not modeled by this approximation
//!
//! # When To Use
//!
//! Use this model when pressure/density variation is negligible and you only
//! need sensible heat effects (e.g. many water heating/storage problems).
//!
//! If you need temperature/pressure dependent properties (or phase change),
//! use [`super::CoolProp`] (when enabled) instead.
//!
//! # Reference State
//!
//! Enthalpy and entropy are reported relative to a configurable reference state
//! (`T_ref`, `ρ_ref`, `h_ref`, `s_ref`).

use std::{convert::Infallible, marker::PhantomData};

use thiserror::Error;
use uom::{
    ConstZero,
    si::{
        f64::{MassDensity, SpecificHeatCapacity, ThermodynamicTemperature},
        thermodynamic_temperature::degree_celsius,
    },
};

use crate::{
    PropertyError, State,
    capability::{
        HasCp, HasCv, HasEnthalpy, HasEntropy, HasInternalEnergy, StateFrom, ThermoModel,
    },
    units::{SpecificEnthalpy, SpecificEntropy, SpecificInternalEnergy, TemperatureDifference},
};

#[derive(Debug, Error, Clone, PartialEq)]
pub enum IncompressibleParametersError {
    #[error("invalid cp: {cp:?}")]
    Cp { cp: SpecificHeatCapacity },
    #[error("invalid reference temperature: {t_ref:?}")]
    ReferenceTemperature { t_ref: ThermodynamicTemperature },
    #[error("invalid reference density: {rho_ref:?}")]
    ReferenceDensity { rho_ref: MassDensity },
}

/// Reference values used to define enthalpy/entropy offsets for an [`Incompressible`] model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncompressibleReference {
    pub temperature: ThermodynamicTemperature,
    pub density: MassDensity,
    pub enthalpy: SpecificEnthalpy,
    pub entropy: SpecificEntropy,
}

impl IncompressibleReference {
    /// Returns a reference with `T_ref = 25°C`, `h_ref = 0`, `s_ref = 0`, and the provided `ρ_ref`.
    #[must_use]
    pub fn standard(density: MassDensity) -> Self {
        Self {
            temperature: ThermodynamicTemperature::new::<degree_celsius>(25.0),
            density,
            enthalpy: SpecificEnthalpy::ZERO,
            entropy: SpecificEntropy::ZERO,
        }
    }
}

/// Constant parameters for the [`Incompressible`] model.
///
/// These values are typically provided by a fluid's [`IncompressibleFluid`] implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IncompressibleParameters {
    pub cp: SpecificHeatCapacity,
    pub reference: IncompressibleReference,
}

impl IncompressibleParameters {
    #[must_use]
    pub fn new(cp: SpecificHeatCapacity, reference_density: MassDensity) -> Self {
        Self {
            cp,
            reference: IncompressibleReference::standard(reference_density),
        }
    }

    #[must_use]
    pub fn with_reference(mut self, reference: IncompressibleReference) -> Self {
        self.reference = reference;
        self
    }
}

/// Fluid constants required by the [`Incompressible`] model.
pub trait IncompressibleFluid {
    /// Returns the constant parameters for use with [`Incompressible`].
    fn parameters() -> IncompressibleParameters;
}

/// Incompressible liquid model with constant density and constant heat capacity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Incompressible<Fluid> {
    cp: SpecificHeatCapacity,
    t_ref: ThermodynamicTemperature,
    rho_ref: MassDensity,
    h_ref: SpecificEnthalpy,
    s_ref: SpecificEntropy,
    _marker: PhantomData<Fluid>,
}

impl<Fluid> ThermoModel for Incompressible<Fluid> {
    type Fluid = Fluid;
}

impl<Fluid> Incompressible<Fluid> {
    /// Creates an incompressible model using constants defined by `Fluid`.
    ///
    /// # Errors
    ///
    /// Returns [`IncompressibleParametersError`] if any required constant is invalid.
    pub fn new() -> Result<Self, IncompressibleParametersError>
    where
        Fluid: IncompressibleFluid,
    {
        let parameters = Fluid::parameters();
        let cp = parameters.cp;
        if !is_strictly_positive(cp.value) {
            return Err(IncompressibleParametersError::Cp { cp });
        }

        let t_ref = parameters.reference.temperature;
        if !is_strictly_positive(t_ref.value) {
            return Err(IncompressibleParametersError::ReferenceTemperature { t_ref });
        }

        let rho_ref = parameters.reference.density;
        if !is_strictly_positive(rho_ref.value) {
            return Err(IncompressibleParametersError::ReferenceDensity { rho_ref });
        }

        Ok(Self {
            cp,
            t_ref,
            rho_ref,
            h_ref: parameters.reference.enthalpy,
            s_ref: parameters.reference.entropy,
            _marker: PhantomData,
        })
    }

    /// Returns the constant reference density used by this model.
    #[must_use]
    pub fn reference_density(&self) -> MassDensity {
        self.rho_ref
    }

    /// Creates a state at the fluid's reference temperature and density.
    #[must_use]
    pub fn reference_state(&self, fluid: Fluid) -> State<Fluid> {
        let temperature = self.t_ref;
        let density = self.rho_ref;

        State {
            temperature,
            density,
            fluid,
        }
    }
}

impl<Fluid> HasInternalEnergy for Incompressible<Fluid> {
    /// Computes internal energy, which is equal to enthalpy for incompressible fluids.
    fn internal_energy(
        &self,
        state: &State<Fluid>,
    ) -> Result<SpecificInternalEnergy, PropertyError> {
        self.enthalpy(state)
    }
}

impl<Fluid> HasEnthalpy for Incompressible<Fluid> {
    /// Computes enthalpy using `h = h₀ + c·(T − T₀)`.
    fn enthalpy(&self, state: &State<Fluid>) -> Result<SpecificEnthalpy, PropertyError> {
        let c = self.cp;
        let t_ref = self.t_ref;
        let h_ref = self.h_ref;

        Ok(h_ref + c * state.temperature.minus(t_ref))
    }
}

impl<Fluid> HasEntropy for Incompressible<Fluid> {
    /// Computes entropy with `s = s₀ + c·ln(T/T₀)`.
    fn entropy(&self, state: &State<Fluid>) -> Result<SpecificEntropy, PropertyError> {
        let c = self.cp;
        let t_ref = self.t_ref;
        let s_ref = self.s_ref;

        Ok(s_ref + c * (state.temperature / t_ref).ln())
    }
}

impl<Fluid> HasCp for Incompressible<Fluid> {
    /// Returns the constant specific heat of the fluid.
    fn cp(&self, _state: &State<Fluid>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(self.cp)
    }
}

impl<Fluid> HasCv for Incompressible<Fluid> {
    /// Returns the constant specific heat of the fluid.
    fn cv(&self, _state: &State<Fluid>) -> Result<SpecificHeatCapacity, PropertyError> {
        Ok(self.cp)
    }
}

/// Enables state creation from temperature alone.
///
/// The returned state uses the fluid's reference density.
impl<Fluid> StateFrom<(Fluid, ThermodynamicTemperature)> for Incompressible<Fluid> {
    type Error = Infallible;

    fn state_from(
        &self,
        (fluid, temperature): (Fluid, ThermodynamicTemperature),
    ) -> Result<State<Fluid>, Self::Error> {
        let density = self.rho_ref;

        Ok(State {
            temperature,
            density,
            fluid,
        })
    }
}

impl<Fluid: Default> StateFrom<ThermodynamicTemperature> for Incompressible<Fluid> {
    type Error = Infallible;

    fn state_from(
        &self,
        temperature: ThermodynamicTemperature,
    ) -> Result<State<Fluid>, Self::Error> {
        self.state_from((Fluid::default(), temperature))
    }
}

/// Returns `true` if `value` is not `NaN` and greater than zero.
fn is_strictly_positive(value: f64) -> bool {
    !value.is_nan() && value > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassDensity, SpecificHeatCapacity, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        specific_heat_capacity::kilojoule_per_kilogram_kelvin,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::capability::HasCp;
    use crate::units::TemperatureDifference;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct MockLiquid;

    impl IncompressibleFluid for MockLiquid {
        fn parameters() -> IncompressibleParameters {
            IncompressibleParameters::new(
                SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(10.0),
                MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            )
        }
    }

    fn mock_liquid_model() -> Incompressible<MockLiquid> {
        Incompressible::<MockLiquid>::new()
            .expect("mock liquid parameters must be physically valid")
    }

    #[test]
    fn internal_energy_equals_enthalpy() -> Result<(), PropertyError> {
        let thermo = mock_liquid_model();
        let fluid = MockLiquid;

        // State at specified temperature and reference density.
        let state: State<MockLiquid> = thermo
            .state_from((fluid, ThermodynamicTemperature::new::<degree_celsius>(15.0)))
            .unwrap();

        let u = thermo.internal_energy(&state)?;
        let h = thermo.enthalpy(&state)?;
        assert_eq!(u, h);

        Ok(())
    }

    #[test]
    fn increase_temperature() -> Result<(), PropertyError> {
        let thermo = mock_liquid_model();

        // State at specified temperature and density.
        let state_a: State<MockLiquid> = State::new(
            ThermodynamicTemperature::new::<degree_celsius>(30.0),
            MassDensity::new::<kilogram_per_cubic_meter>(2.0),
            MockLiquid,
        );

        let state_b =
            state_a.with_temperature(ThermodynamicTemperature::new::<degree_celsius>(60.0));

        // Check that enthalpy increases with temperature using `h = h₀ + c·(T - T₀)`.
        let h_a = thermo.enthalpy(&state_a)?;
        let h_b = thermo.enthalpy(&state_b)?;
        let c = thermo.cp(&state_a)?;
        assert_relative_eq!(
            (h_b - h_a).value,
            (c * state_b.temperature.minus(state_a.temperature)).value,
        );

        // Check that entropy increases with temperature using `s = s₀ + c·ln(T/T₀)`.
        let s_a = thermo.entropy(&state_a)?;
        let s_b = thermo.entropy(&state_b)?;
        assert_relative_eq!(
            (s_b - s_a).value,
            (c * (state_b.temperature / state_a.temperature).ln()).value,
            epsilon = 1e-10,
        );

        Ok(())
    }
}
