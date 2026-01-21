//! Resolution of boundary conditions to determine heat exchanger endpoints.

use twine_thermo::{
    State,
    units::{SpecificEnthalpy, TemperatureDifference},
};
use uom::si::f64::{MassRate, Power, Pressure, ThermodynamicTemperature};

use crate::thermal::hx::discretized::{
    Given, HeatTransferRate, Known, traits::DiscretizedHxThermoModel,
};

use super::SolveError;

/// Resolved boundary conditions and endpoint states for a discretized heat exchanger.
pub struct Resolved<TopFluid, BottomFluid> {
    pub top: ResolvedStream<TopFluid>,
    pub bottom: ResolvedStream<BottomFluid>,
    pub q_dot: HeatTransferRate,
}

/// Stream endpoints and derived values after resolution.
pub struct ResolvedStream<Fluid> {
    pub inlet: State<Fluid>,
    pub outlet: State<Fluid>,
    pub h_in: SpecificEnthalpy,
    pub p_in: Pressure,
    pub p_out: Pressure,
    pub m_dot: MassRate,
}

impl<TopFluid: Clone, BottomFluid: Clone> Resolved<TopFluid, BottomFluid> {
    pub fn new(
        known: &Known<TopFluid, BottomFluid>,
        given: Given,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Self, SolveError> {
        let context = ResolveContext::from_known(known, thermo_top, thermo_bottom)?;
        match given {
            Given::TopOutletTemp(t_out) => {
                context.resolve_from_top_outlet(t_out, thermo_top, thermo_bottom)
            }
            Given::BottomOutletTemp(t_out) => {
                context.resolve_from_bottom_outlet(t_out, thermo_top, thermo_bottom)
            }
            Given::HeatTransferRate(q_dot) => {
                context.resolve_from_heat_transfer_rate(q_dot, thermo_top, thermo_bottom)
            }
        }
    }
}

/// Resolution context holding inlet data and computed outlet pressures.
///
/// This encapsulates the common setup work shared by all [`Given`] variants.
struct ResolveContext<TopFluid, BottomFluid> {
    top: StreamContext<TopFluid>,
    bottom: StreamContext<BottomFluid>,
}

/// Stream inputs and derived values used during resolution.
struct StreamContext<Fluid> {
    inlet: State<Fluid>,
    fluid: Fluid,
    h_in: SpecificEnthalpy,
    p_in: Pressure,
    p_out: Pressure,
    m_dot: MassRate,
}

impl<Fluid> StreamContext<Fluid> {
    fn into_resolved(self, outlet: State<Fluid>) -> ResolvedStream<Fluid> {
        ResolvedStream {
            inlet: self.inlet,
            outlet,
            h_in: self.h_in,
            p_in: self.p_in,
            p_out: self.p_out,
            m_dot: self.m_dot,
        }
    }
}

impl<TopFluid: Clone, BottomFluid: Clone> ResolveContext<TopFluid, BottomFluid> {
    /// Extracts and caches inlet properties from [`Known`] inputs.
    ///
    /// This performs all the common setup work: extracting inlet pressures and
    /// enthalpies, computing outlet pressures from pressure drops, and storing
    /// mass flow rates.
    fn from_known(
        known: &Known<TopFluid, BottomFluid>,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Self, SolveError> {
        let top_in = known.inlets.top.clone();
        let bottom_in = known.inlets.bottom.clone();
        let m_dot_top = known.m_dot.top();
        let m_dot_bottom = known.m_dot.bottom();

        let p_top_in = thermo_top
            .pressure(&top_in)
            .map_err(|err| SolveError::thermo_failed("pressure(top inlet)", err))?;
        let p_bottom_in = thermo_bottom
            .pressure(&bottom_in)
            .map_err(|err| SolveError::thermo_failed("pressure(bottom inlet)", err))?;
        let p_top_out = p_top_in - known.dp.top();
        let p_bottom_out = p_bottom_in - known.dp.bottom();

        let h_top_in = thermo_top
            .enthalpy(&top_in)
            .map_err(|err| SolveError::thermo_failed("enthalpy(top inlet)", err))?;
        let h_bottom_in = thermo_bottom
            .enthalpy(&bottom_in)
            .map_err(|err| SolveError::thermo_failed("enthalpy(bottom inlet)", err))?;

        let top_fluid = top_in.fluid.clone();
        let bottom_fluid = bottom_in.fluid.clone();

        Ok(Self {
            top: StreamContext {
                inlet: top_in,
                fluid: top_fluid,
                h_in: h_top_in,
                p_in: p_top_in,
                p_out: p_top_out,
                m_dot: m_dot_top,
            },
            bottom: StreamContext {
                inlet: bottom_in,
                fluid: bottom_fluid,
                h_in: h_bottom_in,
                p_in: p_bottom_in,
                p_out: p_bottom_out,
                m_dot: m_dot_bottom,
            },
        })
    }

    /// Resolves outlets given the top stream outlet temperature.
    fn resolve_from_top_outlet(
        self,
        t_out: ThermodynamicTemperature,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Resolved<TopFluid, BottomFluid>, SolveError> {
        let Self { top, bottom } = self;

        let top_out = thermo_top
            .state_from((top.fluid.clone(), t_out, top.p_out))
            .map_err(|err| SolveError::thermo_failed("state_from(top outlet)", err))?;

        let h_top_out = thermo_top
            .enthalpy(&top_out)
            .map_err(|err| SolveError::thermo_failed("enthalpy(top outlet)", err))?;

        let q_signed = top.m_dot * (top.h_in - h_top_out);
        let q_dot = heat_transfer_rate_from_signed(
            top.inlet.temperature,
            bottom.inlet.temperature,
            Some(top_out.temperature),
            None,
            q_signed,
        )?;

        let h_bottom_out = bottom.h_in + q_signed / bottom.m_dot;
        let bottom_out = thermo_bottom
            .state_from((bottom.fluid.clone(), bottom.p_out, h_bottom_out))
            .map_err(|err| SolveError::thermo_failed("state_from(bottom outlet)", err))?;

        Ok(Resolved {
            top: top.into_resolved(top_out),
            bottom: bottom.into_resolved(bottom_out),
            q_dot,
        })
    }

    /// Resolves outlets given the bottom stream outlet temperature.
    fn resolve_from_bottom_outlet(
        self,
        t_out: ThermodynamicTemperature,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Resolved<TopFluid, BottomFluid>, SolveError> {
        let Self { top, bottom } = self;

        let bottom_out = thermo_bottom
            .state_from((bottom.fluid.clone(), t_out, bottom.p_out))
            .map_err(|err| SolveError::thermo_failed("state_from(bottom outlet)", err))?;

        let h_bottom_out = thermo_bottom
            .enthalpy(&bottom_out)
            .map_err(|err| SolveError::thermo_failed("enthalpy(bottom outlet)", err))?;

        let q_signed = bottom.m_dot * (h_bottom_out - bottom.h_in);
        let q_dot = heat_transfer_rate_from_signed(
            top.inlet.temperature,
            bottom.inlet.temperature,
            None,
            Some(bottom_out.temperature),
            q_signed,
        )?;

        let h_top_out = top.h_in - q_signed / top.m_dot;
        let top_out = thermo_top
            .state_from((top.fluid.clone(), top.p_out, h_top_out))
            .map_err(|err| SolveError::thermo_failed("state_from(top outlet)", err))?;

        Ok(Resolved {
            top: top.into_resolved(top_out),
            bottom: bottom.into_resolved(bottom_out),
            q_dot,
        })
    }

    /// Resolves outlets given the heat transfer rate.
    fn resolve_from_heat_transfer_rate(
        self,
        q_dot: HeatTransferRate,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Resolved<TopFluid, BottomFluid>, SolveError> {
        let Self { top, bottom } = self;

        let q_signed = q_dot.signed_top_to_bottom();
        let h_top_out = top.h_in - q_signed / top.m_dot;
        let h_bottom_out = bottom.h_in + q_signed / bottom.m_dot;

        let top_out = thermo_top
            .state_from((top.fluid.clone(), top.p_out, h_top_out))
            .map_err(|err| SolveError::thermo_failed("state_from(top outlet)", err))?;
        let bottom_out = thermo_bottom
            .state_from((bottom.fluid.clone(), bottom.p_out, h_bottom_out))
            .map_err(|err| SolveError::thermo_failed("state_from(bottom outlet)", err))?;

        Ok(Resolved {
            top: top.into_resolved(top_out),
            bottom: bottom.into_resolved(bottom_out),
            q_dot,
        })
    }
}

/// Builds a heat transfer rate and converts NaN inputs into a Second Law violation.
fn heat_transfer_rate_from_signed(
    top_inlet_temp: ThermodynamicTemperature,
    bottom_inlet_temp: ThermodynamicTemperature,
    top_outlet_temp: Option<ThermodynamicTemperature>,
    bottom_outlet_temp: Option<ThermodynamicTemperature>,
    q_signed: Power,
) -> Result<HeatTransferRate, SolveError> {
    HeatTransferRate::from_signed_top_to_bottom(q_signed).map_err(|_| {
        let min_delta_t = top_inlet_temp.minus(bottom_inlet_temp);
        SolveError::SecondLawViolation {
            top_outlet_temp,
            bottom_outlet_temp,
            q_dot: q_signed,
            min_delta_t,
            violation_node: None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassRate, ThermodynamicTemperature},
        mass_rate::kilogram_per_second,
        power::kilowatt,
        thermodynamic_temperature::kelvin,
    };

    use crate::thermal::hx::discretized::{
        Inlets, MassFlows, PressureDrops,
        test_support::{TestThermoModel, state},
    };

    #[test]
    fn resolve_top_outlet_closes_energy_balance() {
        let model = TestThermoModel::new();

        let known = Known {
            inlets: Inlets {
                top: state(350.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(1.0),
                MassRate::new::<kilogram_per_second>(1.0),
            ),
            dp: PressureDrops::default(),
        };

        let resolved = Resolved::new(
            &known,
            Given::TopOutletTemp(ThermodynamicTemperature::new::<kelvin>(330.0)),
            &model,
            &model,
        )
        .expect("resolution should succeed");

        assert_relative_eq!(
            resolved.q_dot.signed_top_to_bottom().get::<kilowatt>(),
            20.0,
        );
        assert_relative_eq!(resolved.bottom.outlet.temperature.get::<kelvin>(), 320.0);
    }
}
