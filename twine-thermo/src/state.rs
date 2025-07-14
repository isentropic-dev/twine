use std::marker::PhantomData;

use uom::si::f64::{MassDensity, ThermodynamicTemperature};

/// Represents the thermodynamic state of a fluid.
///
/// A `State<Fluid, Model>` captures the thermodynamic state of a specific fluid
/// using a specific thermodynamic model, including its temperature, density,
/// and any fluid-specific data.
///
/// The `Fluid` type parameter can be a simple marker type, such as [`Air`] or
/// [`Water`], or a structured type containing additional data, such as mixture
/// composition or particle concentrations.
///
/// The `Model` type parameter ensures compile-time safety by preventing states
/// from being used with incompatible thermodynamic models.
/// This type-level enforcement eliminates runtime model validation overhead
/// and helps prevent physically inconsistent results that could arise from
/// inadvertently applying different models to the same temperature and density
/// of a given fluid.
///
/// `State` is the primary input to [`ThermodynamicProperties`] models for
/// calculating pressure, enthalpy, entropy, and related quantities.
///
/// # Example
///
/// ```
/// use twine_thermo::{State, fluid::Air, model::ideal_gas::IdealGas};
/// use uom::si::{
///     f64::{ThermodynamicTemperature, MassDensity},
///     thermodynamic_temperature::kelvin,
///     mass_density::kilogram_per_cubic_meter,
/// };
///
/// let state: State<Air, IdealGas> = State::new(
///     ThermodynamicTemperature::new::<kelvin>(300.0),
///     MassDensity::new::<kilogram_per_cubic_meter>(1.0),
///     Air,
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct State<Fluid, Model> {
    pub temperature: ThermodynamicTemperature,
    pub density: MassDensity,
    pub fluid: Fluid,
    _marker: PhantomData<Model>,
}

impl<Fluid, Model> State<Fluid, Model> {
    /// Creates a new state with the given temperature, density, and fluid.
    #[must_use]
    pub fn new(temperature: ThermodynamicTemperature, density: MassDensity, fluid: Fluid) -> Self {
        Self {
            temperature,
            density,
            fluid,
            _marker: PhantomData,
        }
    }

    /// Returns a new state with the given temperature, keeping other fields unchanged.
    #[must_use]
    pub fn with_temperature(self, temperature: ThermodynamicTemperature) -> Self {
        Self {
            temperature,
            ..self
        }
    }

    /// Returns a new state with the given density, keeping other fields unchanged.
    #[must_use]
    pub fn with_density(self, density: MassDensity) -> Self {
        Self { density, ..self }
    }

    /// Returns a new state with the given fluid, keeping other fields unchanged.
    #[must_use]
    pub fn with_fluid(self, fluid: Fluid) -> Self {
        Self { fluid, ..self }
    }
}
