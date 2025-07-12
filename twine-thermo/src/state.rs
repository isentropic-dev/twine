use uom::si::f64::{MassDensity, ThermodynamicTemperature};

/// Represents the thermodynamic state of a fluid.
///
/// A `State<Fluid>` captures the instantaneous thermodynamic conditions of a
/// specific fluid, including temperature, density, and any fluid-specific data.
///
/// The `fluid` field can be a simple marker type, such as [`Air`] or [`Water`],
/// or a structured type containing additional data, such as mixture composition
/// or particle concentrations.
///
/// `State` is the primary input to [`ThermodynamicProperties`] models for
/// calculating pressure, enthalpy, entropy, and related quantities.
///
/// # Example
///
/// ```
/// use twine_thermo::State;
/// use uom::si::{
///     f64::{ThermodynamicTemperature, MassDensity},
///     thermodynamic_temperature::kelvin,
///     mass_density::kilogram_per_cubic_meter,
/// };
///
/// struct Air;
///
/// let state = State {
///     temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
///     density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
///     fluid: Air,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct State<Fluid> {
    pub temperature: ThermodynamicTemperature,
    pub density: MassDensity,
    pub fluid: Fluid,
}

impl<Fluid> State<Fluid> {
    /// Creates a new state with the given temperature, density, and fluid.
    #[must_use]
    pub fn new(temperature: ThermodynamicTemperature, density: MassDensity, fluid: Fluid) -> Self {
        Self {
            temperature,
            density,
            fluid,
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
