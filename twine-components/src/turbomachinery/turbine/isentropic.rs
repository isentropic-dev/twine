//! Isentropic turbine model.
//!
//! Computes an outlet state at a target pressure using an isentropic efficiency `eta`.
//!
//! Given an inlet state and a target outlet pressure, the model:
//! 1. computes an ideal (isentropic) outlet at constant entropy: `(p_out, s_in)`,
//! 2. evaluates the ideal enthalpy drop `Δh_s = h_in − h_out,s`,
//! 3. maps to an actual enthalpy drop using `η = Δh / Δh_s` ⇒ `Δh = η*Δh_s`,
//! 4. requests an outlet state from `(p_out, h_out)` and reports the produced work.

use twine_core::constraint::{Constrained, UnitInterval};
use twine_thermo::{
    State,
    model::{StateFrom, ThermodynamicProperties},
    units::{SpecificEnthalpy, SpecificEntropy},
};
use uom::si::f64::{Pressure, Ratio};

use crate::turbomachinery::{
    turbine::{ExpansionError, ExpansionResult},
    work::ExpansionWork,
};

/// Computes the turbine outlet state and produced work using an isentropic efficiency model.
///
/// `eta` is an isentropic efficiency constrained to `[0, 1]`.
/// Values near zero represent extremely inefficient expansion and yield little
/// to no produced work.
///
/// The returned [`ExpansionResult::work`] is the target enthalpy drop
/// `h_in − h_out`, reported as an [`ExpansionWork`]. If the thermodynamic
/// model returns the outlet state via inverse-solving or interpolation,
/// `thermo.enthalpy(&outlet)` may differ slightly from the target.
///
/// # Errors
///
/// Returns [`ExpansionError`] if the thermodynamic model fails, `p_out > p_in`,
/// or the resulting work is non-physical.
pub fn isentropic<Fluid, M>(
    inlet: &State<Fluid>,
    p_out: Pressure,
    eta: Constrained<Ratio, UnitInterval>,
    thermo: &M,
) -> Result<ExpansionResult<Fluid>, ExpansionError<Fluid>>
where
    M: ThermodynamicProperties<Fluid>
        + StateFrom<Fluid, (Pressure, SpecificEntropy)>
        + StateFrom<Fluid, (Pressure, SpecificEnthalpy)>,
{
    let p_in = thermo
        .pressure(inlet)
        .map_err(ExpansionError::inlet_pressure_failed)?;

    let h_in = thermo
        .enthalpy(inlet)
        .map_err(ExpansionError::inlet_enthalpy_failed)?;

    let s_in = thermo
        .entropy(inlet)
        .map_err(ExpansionError::inlet_entropy_failed)?;

    isentropic_core(p_in, h_in, s_in, p_out, eta, thermo)
}

/// Core isentropic expansion model.
///
/// This function assumes `p_in`, `h_in`, and `s_in` are internally consistent
/// and were produced using the provided `thermo` model instance.
///
/// # Errors
///
/// Returns [`ExpansionError`] if the thermodynamic model fails, `p_out > p_in`,
/// or the resulting work is non-physical.
pub(crate) fn isentropic_core<Fluid, M>(
    p_in: Pressure,
    h_in: SpecificEnthalpy,
    s_in: SpecificEntropy,
    p_out: Pressure,
    eta: Constrained<Ratio, UnitInterval>,
    thermo: &M,
) -> Result<ExpansionResult<Fluid>, ExpansionError<Fluid>>
where
    M: ThermodynamicProperties<Fluid>
        + StateFrom<Fluid, (Pressure, SpecificEntropy)>
        + StateFrom<Fluid, (Pressure, SpecificEnthalpy)>,
{
    if p_out > p_in {
        return Err(ExpansionError::OutletPressureGreaterThanInlet { p_in, p_out });
    }

    let isentropic_outlet = thermo.state_from((p_out, s_in)).map_err(|source| {
        ExpansionError::ideal_outlet_state_from_pressure_entropy_failed(p_out, s_in, source)
    })?;

    let h_out_s = thermo
        .enthalpy(&isentropic_outlet)
        .map_err(ExpansionError::ideal_outlet_enthalpy_failed)?;

    let dh_s = h_in - h_out_s;
    let dh_actual = dh_s * eta.into_inner();
    let h_out_target = h_in - dh_actual;

    let outlet = thermo.state_from((p_out, h_out_target)).map_err(|source| {
        ExpansionError::outlet_state_from_pressure_enthalpy_failed(p_out, h_out_target, source)
    })?;

    let raw_work = h_in - h_out_target;

    match ExpansionWork::new(raw_work) {
        Ok(work) => Ok(ExpansionResult { outlet, work }),
        Err(_) => Err(ExpansionError::NonPhysicalWork { outlet, raw_work }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use twine_core::constraint::UnitInterval;
    use twine_thermo::{PropertyError, model::ideal_gas::IdealGas};
    use uom::si::{
        f64::{MassDensity, Pressure, Ratio, ThermodynamicTemperature},
        mass_density::kilogram_per_cubic_meter,
        pressure::{kilopascal, pascal},
        ratio::ratio,
        specific_heat_capacity::joule_per_kilogram_kelvin,
        thermodynamic_temperature::kelvin,
    };

    use crate::turbomachinery::test_utils::{FakeMode, FakeThermo, MockGas, enth_si};

    #[test]
    fn normal_expansion_matches_expected_work() {
        let thermo = IdealGas;

        let p_in = Pressure::new::<kilopascal>(12_800.0);
        let t_in = ThermodynamicTemperature::new::<kelvin>(1200.0);
        let inlet: State<MockGas> = thermo.state_from((t_in, p_in)).unwrap();

        // Pick `p2/p1 = 1/2^7` so that `T2s = T1*(p2/p1)^((k-1)/k) = T1*(1/2^7)^(2/7) = T1/4`.
        let p_out = Pressure::new::<kilopascal>(100.0);
        let eta = UnitInterval::new(Ratio::new::<ratio>(0.9)).unwrap();

        let result = isentropic(&inlet, p_out, eta, &thermo).unwrap();

        // Expected output for an ideal gas:
        // ```
        // k = cp/(cp-R) = 1.4
        // T2s = T1/4 = 300 K
        // w = eta*cp*(T1 - T2s) = 810_000 J/kg
        // ```
        let expected_work_j_per_kg = 810_000.0;
        assert_relative_eq!(result.work.quantity().value, expected_work_j_per_kg);

        // The outlet state is computed from (p_out, h_out_target),
        // so for an ideal gas: `T2 = T1 - w/cp = 1200 K - 810_000/1000 = 390 K`.
        assert_relative_eq!(
            result.outlet.temperature.get::<kelvin>(),
            390.0,
            epsilon = 1e-12,
        );
    }

    #[test]
    fn outlet_pressure_greater_than_inlet_is_an_error() {
        let inlet = State {
            temperature: ThermodynamicTemperature::new::<kelvin>(300.0),
            density: MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            fluid: MockGas,
        };
        let p_in = IdealGas.pressure(&inlet).unwrap();
        let p_out = p_in + Pressure::new::<pascal>(1.0);

        let err = isentropic(&inlet, p_out, UnitInterval::one(), &IdealGas).unwrap_err();

        match err {
            ExpansionError::OutletPressureGreaterThanInlet {
                p_in: reported_p_in,
                p_out: reported_p_out,
            } => {
                assert_eq!(reported_p_in, p_in);
                assert_eq!(reported_p_out, p_out);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    // Shared inputs for `isentropic_core` error-path tests.
    fn core_inputs() -> (
        Pressure,
        SpecificEnthalpy,
        SpecificEntropy,
        Pressure,
        Constrained<Ratio, UnitInterval>,
    ) {
        let p_in = Pressure::new::<kilopascal>(200.0);
        let h_in = enth_si(0.0);
        let s_in = SpecificEntropy::new::<joule_per_kilogram_kelvin>(0.0);
        let p_out = Pressure::new::<kilopascal>(100.0);
        let eta = UnitInterval::one();
        (p_in, h_in, s_in, p_out, eta)
    }

    #[test]
    fn core_state_from_pressure_entropy_failure_is_wrapped() {
        let thermo = FakeThermo {
            mode: FakeMode::FailStateFromPressureEntropy,
        };
        let (p_in, h_in, s_in, p_out, eta) = core_inputs();

        let err = isentropic_core(p_in, h_in, s_in, p_out, eta, &thermo).unwrap_err();

        match err {
            ExpansionError::ThermodynamicModelFailed { context, source: _ } => {
                assert!(context.contains("ideal_outlet_state_from("));
                assert!(context.contains("p_out="));
                assert!(context.contains("s_in="));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn core_state_from_pressure_enthalpy_failure_is_wrapped() {
        let thermo = FakeThermo {
            mode: FakeMode::FailStateFromPressureEnthalpy,
        };
        let (p_in, h_in, s_in, p_out, eta) = core_inputs();

        let err = isentropic_core(p_in, h_in, s_in, p_out, eta, &thermo).unwrap_err();

        match err {
            ExpansionError::ThermodynamicModelFailed { context, source: _ } => {
                assert!(context.contains("outlet_state_from("));
                assert!(context.contains("p_out="));
                assert!(context.contains("h_out_target="));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn core_property_failure_is_wrapped() {
        let thermo = FakeThermo {
            mode: FakeMode::FailEnthalpy,
        };
        let (p_in, h_in, s_in, p_out, eta) = core_inputs();

        let err = isentropic_core(p_in, h_in, s_in, p_out, eta, &thermo).unwrap_err();

        match err {
            ExpansionError::ThermodynamicModelFailed { context, source } => {
                assert_eq!(context, "enthalpy(ideal outlet)");
                let source = source
                    .downcast_ref::<PropertyError>()
                    .expect("expected PropertyError source");
                assert!(matches!(source, PropertyError::Calculation(_)));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn core_non_physical_work_returns_outlet_state() {
        let thermo = FakeThermo {
            mode: FakeMode::FixedEnthalpy(enth_si(1.0)),
        };
        let (p_in, h_in, s_in, p_out, eta) = core_inputs();

        let err = isentropic_core(p_in, h_in, s_in, p_out, eta, &thermo).unwrap_err();

        match err {
            ExpansionError::NonPhysicalWork {
                outlet: _,
                raw_work,
                ..
            } => {
                assert!(raw_work.value < 0.0);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
