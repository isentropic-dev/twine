use twine_core::{TimeDerivative, TimeIntegrable};
use uom::si::f64::{MassDensity, ThermodynamicTemperature, Time};

/// Represents the thermodynamic state of a fluid.
///
/// A `State<Fluid>` captures the instantaneous thermodynamic conditions of a
/// system, including temperature, density, and fluid-specific data.
///
/// The `fluid` field may be a unit-like marker type (such as [`Air`] or [`Water`]),
/// or it may carry additional state (such as composition fractions for a mixture
/// or a particle concentration), depending on the modeling context.
///
/// `State` is the primary input to [`ThermodynamicProperties`] models for
/// calculating pressure, enthalpy, entropy, and related quantities.
///
/// The `Fluid` type parameter is generic and must implement [`TimeIntegrable`],
/// allowing the state to evolve over time through time-stepping integration.
///
/// # Example
///
/// ```
/// use twine_thermo::{fluids::Air, State};
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
#[derive(Debug, Clone, PartialEq)]
pub struct State<Fluid: TimeIntegrable> {
    pub temperature: ThermodynamicTemperature,
    pub density: MassDensity,
    pub fluid: Fluid,
}

/// Time derivative of a [`State`].
///
/// Represents the rate of change of a fluid state with respect to time,
/// including derivatives of temperature, density, and any fluid-specific data.
///
/// The `fluid` derivative is often `()` for static-property fluids like [`Air`]
/// or [`Water`], but may carry data such as rates of change for species mass
/// fractions (in a mixture), particle concentrations, or other internal state.
#[derive(Debug, Clone, PartialEq)]
pub struct StateDerivative<Fluid: TimeIntegrable> {
    pub temperature: TimeDerivative<ThermodynamicTemperature>,
    pub density: TimeDerivative<MassDensity>,
    pub fluid: TimeDerivative<Fluid>,
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
