use twine_thermo::{
    capability::ThermoModel,
    units::{SpecificEnthalpy, SpecificEntropy},
};
use uom::si::f64::Pressure;

/// Inlet thermodynamic properties tied to a specific model instance.
///
/// This is a small bundle used by turbomachinery core routines to ensure that:
/// - inlet-derived scalars (e.g. `p_in`, `h_in`, `s_in`) remain consistent with each other, and
/// - the inverse state construction (`state_from`) uses the same model instance and fluid that
///   produced those scalars.
///
/// This type is intentionally device-agnostic (compressor/turbine/polytropic/head-based
/// models can all use it) and only contains inlet-derived quantities.
#[derive(Debug)]
pub(crate) struct InletProperties<'a, F, M>
where
    M: ThermoModel<Fluid = F>,
{
    pub(crate) thermo: &'a M,
    pub(crate) fluid: F,
    pub(crate) p_in: Pressure,
    pub(crate) h_in: SpecificEnthalpy,
    pub(crate) s_in: SpecificEntropy,
}
