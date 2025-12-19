use crate::State;

use super::ThermoModel;

/// Capability for constructing a [`State`] from a typed input.
///
/// A thermodynamic [`State`] includes a `Fluid` value. In Twine, `Fluid` is
/// allowed to carry *state-defining* information such as mixture composition,
/// salinity, or any other configuration needed to make the state well-defined.
///
/// `StateFrom<Input>` expresses, at compile time, which combinations of
/// inputs a model can use to construct a state.
/// If a model does not implement `StateFrom<Input>`, then that input is simply
/// not supported (no runtime "not implemented" errors).
///
/// ## Common input patterns
///
/// Inputs are intentionally represented as normal Rust types (often tuples).
/// Common patterns include:
/// - `(Fluid, ThermodynamicTemperature, Pressure)` (temperature + pressure)
/// - `(Fluid, Pressure, MassDensity)` (pressure + density)
/// - `(Fluid, Pressure, SpecificEnthalpy)` (pressure + enthalpy)
/// - `(Fluid, Pressure, SpecificEntropy)` (pressure + entropy)
/// - `(Fluid, ThermodynamicTemperature)` (e.g. for an incompressible liquid)
///
/// ## Default fluid convenience
///
/// Many call sites don't want to spell the fluid value when the `Fluid` is a
/// simple marker type (or otherwise has no state-defining data).
/// To support that ergonomically while keeping the core API explicit,
/// we provide blanket implementations that create the fluid using `Fluid::default()`.
///
/// For example, if a model implements `StateFrom<(Fluid, A, B)>` and `Fluid:
/// Default`, then it also implements `StateFrom<(A, B)>`.
/// This enables calls like `thermo.state_from((t, p))` without losing the
/// ability to pass an explicit `Fluid` when the fluid carries state-defining
/// data (composition, salinity, etc.).
pub trait StateFrom<Input>: ThermoModel {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a thermodynamic state from the provided input.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] if the state cannot be created from `input`.
    fn state_from(&self, input: Input) -> Result<State<Self::Fluid>, Self::Error>;
}

/// Default-fluid convenience impl.
///
/// If a model can construct a state from an explicit `(Fluid, A, B)` input and
/// `Fluid: Default`, then it can also construct a state from just `(A, B)` by
/// using `Fluid::default()`.
///
/// This is intended for cases where `Fluid` is a marker type (or otherwise does
/// not carry state-defining data). If the `Fluid` carries state-defining data,
/// prefer inputs that include the explicit fluid value.
impl<M, A, B> StateFrom<(A, B)> for M
where
    M: ThermoModel + StateFrom<(<M as ThermoModel>::Fluid, A, B)>,
    <M as ThermoModel>::Fluid: Default,
{
    type Error = <M as StateFrom<(<M as ThermoModel>::Fluid, A, B)>>::Error;

    fn state_from(&self, (a, b): (A, B)) -> Result<State<Self::Fluid>, Self::Error> {
        self.state_from((<M as ThermoModel>::Fluid::default(), a, b))
    }
}
