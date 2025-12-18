use twine_core::{TimeDerivative, TimeIntegrable};
use uom::si::f64::{MassDensity, ThermodynamicTemperature, Time};

/// The thermodynamic state of a fluid.
///
/// A `State<Fluid>` captures the thermodynamic state of a specific fluid,
/// including its temperature, density, and any fluid-specific data.
///
/// The `Fluid` type parameter can be a simple marker type, such as [`Air`] or
/// [`Water`], or a structured type containing additional data, such as mixture
/// composition or particle concentration.
///
/// `State` is the primary input to capability-based thermodynamic models for
/// calculating pressure, enthalpy, entropy, and related quantities.
///
/// # Example
///
/// ```
/// use twine_thermo::{State, fluid::Air};
/// use uom::si::{
///     f64::{ThermodynamicTemperature, MassDensity},
///     thermodynamic_temperature::kelvin,
///     mass_density::kilogram_per_cubic_meter,
/// };
///
/// let state = State {
///     temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
///     density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
///     fluid: Air,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// The time derivative of a fluid's thermodynamic state.
///
/// A `StateDerivative<Fluid>` represents the instantaneous rate of change of a
/// thermodynamic state, including the time derivatives of temperature, density,
/// and any fluid-specific data that varies over time.
#[derive(Debug, Clone, PartialEq)]
pub struct StateDerivative<Fluid: TimeIntegrable> {
    pub temperature: TimeDerivative<ThermodynamicTemperature>,
    pub density: TimeDerivative<MassDensity>,
    pub fluid: TimeDerivative<Fluid>,
}

impl<Fluid> Copy for StateDerivative<Fluid>
where
    Fluid: TimeIntegrable,
    TimeDerivative<Fluid>: Copy,
{
}

impl<Fluid: TimeIntegrable> TimeIntegrable for State<Fluid> {
    type Derivative = StateDerivative<Fluid>;

    fn step(self, derivative: Self::Derivative, dt: Time) -> Self {
        Self {
            temperature: self.temperature.step(derivative.temperature, dt),
            density: self.density.step(derivative.density, dt),
            fluid: self.fluid.step(derivative.fluid, dt),
        }
    }
}
