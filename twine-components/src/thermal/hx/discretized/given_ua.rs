//! Iterative solver for target thermal conductance (UA).
//!
//! This module provides iterative solving to match a target UA by varying
//! the top stream outlet temperature until the achieved conductance converges
//! to the desired value.

mod config;
mod error;
mod problem;

pub use config::GivenUaConfig;
pub use error::GivenUaError;

use twine_core::constraint::{Constrained, NonNegative};
use twine_solve::equation::{EvalError, bisection};
use uom::{
    ConstZero,
    si::{
        f64::ThermalConductance, thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::kelvin,
    },
};

use super::{
    Given, HeatTransferRate, Known, Results, SolveError,
    traits::{DiscretizedArrangement, DiscretizedHxThermoModel},
};

use problem::{GivenUaModel, GivenUaProblem};

/// Solves a discretized heat exchanger given a target conductance (UA).
///
/// Uses bisection to iteratively find the top stream outlet temperature that
/// achieves the specified thermal conductance.
///
/// # Errors
///
/// Returns [`GivenUaError`] on non-physical results, thermodynamic model failures,
/// or if the solver fails to converge.
pub(super) fn given_ua<Arrangement, TopFluid, BottomFluid, const N: usize>(
    known: &Known<TopFluid, BottomFluid>,
    target_ua: Constrained<ThermalConductance, NonNegative>,
    config: GivenUaConfig,
    thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
    thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
) -> Result<Results<TopFluid, BottomFluid, N>, GivenUaError>
where
    Arrangement: DiscretizedArrangement + Default,
    TopFluid: Clone,
    BottomFluid: Clone,
{
    const {
        assert!(
            N >= 2,
            "discretized heat exchanger requires at least 2 nodes (inlet and outlet)"
        );
    };

    let target_ua = target_ua.into_inner();

    if target_ua == ThermalConductance::ZERO {
        return Ok(super::DiscretizedHx::<Arrangement, N>::solve(
            known,
            Given::HeatTransferRate(HeatTransferRate::None),
            thermo_top,
            thermo_bottom,
        )?);
    }

    let model = GivenUaModel::<Arrangement, _, _, _, _, N>::new(known, thermo_top, thermo_bottom);

    let problem = GivenUaProblem::new(target_ua);

    let solution = bisection::solve(
        &model,
        &problem,
        [
            known.inlets.top.temperature.get::<kelvin>(),
            known.inlets.bottom.temperature.get::<kelvin>(),
        ],
        &config.bisection(),
        |event: &bisection::Event<'_, _, _>| {
            // When a second law violation occurs, the outlet temperature is outside
            // the feasible region. Guide bisection away by assuming positive residual.
            if let Err(EvalError::Model(SolveError::SecondLawViolation { .. })) = event.result() {
                return Some(bisection::Action::assume_positive());
            }
            None
        },
    )?;

    if solution.status != bisection::Status::Converged {
        return Err(GivenUaError::MaxIters {
            residual: ThermalConductance::new::<watt_per_kelvin>(solution.residual),
            iters: solution.iters,
        });
    }

    Ok(solution.snapshot.output)
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use twine_core::constraint::NonNegative;
    use uom::si::{
        f64::{MassRate, ThermodynamicTemperature},
        mass_rate::kilogram_per_second,
        thermal_conductance::kilowatt_per_kelvin,
        thermodynamic_temperature::kelvin,
    };

    use crate::thermal::hx::{
        arrangement::CounterFlow,
        discretized::{
            DiscretizedHx, Given, HeatTransferRate, Inlets, Known, MassFlows, PressureDrops,
            test_support::{TestThermoModel, state},
        },
    };

    #[test]
    fn roundtrip() {
        let model = TestThermoModel::new();

        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(2.0),
                MassRate::new::<kilogram_per_second>(3.0),
            ),
            dp: PressureDrops::default(),
        };

        let target = DiscretizedHx::<CounterFlow, 5>::solve(
            &known,
            Given::TopOutletTemp(ThermodynamicTemperature::new::<kelvin>(360.0)),
            &model,
            &model,
        )
        .expect("baseline solve should succeed");

        let result = given_ua::<CounterFlow, _, _, 5>(
            &known,
            NonNegative::new(target.ua).unwrap(),
            GivenUaConfig::default(),
            &model,
            &model,
        )
        .expect("ua solve should succeed");

        assert_relative_eq!(
            result.top[4].temperature.get::<kelvin>(),
            target.top[4].temperature.get::<kelvin>(),
            epsilon = 1e-12
        );
    }

    #[test]
    fn zero_returns_no_heat_transfer() {
        let model = TestThermoModel::new();

        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(2.0),
                MassRate::new::<kilogram_per_second>(3.0),
            ),
            dp: PressureDrops::default(),
        };

        let result = given_ua::<CounterFlow, _, _, 5>(
            &known,
            NonNegative::new(ThermalConductance::new::<kilowatt_per_kelvin>(0.0)).unwrap(),
            GivenUaConfig::default(),
            &model,
            &model,
        )
        .expect("zero ua solve should succeed");

        // With zero UA, no heat transfer occurs
        assert_eq!(result.q_dot, HeatTransferRate::None);
        assert_eq!(
            result.ua,
            ThermalConductance::new::<kilowatt_per_kelvin>(0.0)
        );

        // Outlet temperatures should match inlet temperatures
        assert_relative_eq!(result.top[4].temperature.get::<kelvin>(), 400.0);
        assert_relative_eq!(result.bottom[0].temperature.get::<kelvin>(), 300.0);
    }

    #[test]
    fn handles_second_law_violations_during_iteration() {
        let model = TestThermoModel::new();

        // Setup with unbalanced flow rates to create challenging conditions.
        // Bottom stream has much lower flow, so it experiences larger temperature changes.
        // In counterflow, this can easily cause temperature crossover if the solver
        // explores outlet temperatures that are too extreme during iteration.
        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(2.0),
                MassRate::new::<kilogram_per_second>(0.5), // 4x imbalance
            ),
            dp: PressureDrops::default(),
        };

        // Request a moderate UA. During bisection search, some trial outlet temperatures
        // will cause the bottom stream to heat beyond the top inlet (temperature crossover).
        // The bisection solver should handle these violations gracefully via the
        // assume_positive guidance logic and converge to a valid solution.
        let result = given_ua::<CounterFlow, _, _, 5>(
            &known,
            NonNegative::new(ThermalConductance::new::<kilowatt_per_kelvin>(2.0)).unwrap(),
            GivenUaConfig::default(),
            &model,
            &model,
        )
        .expect("solver should converge despite violations during iteration");

        // Verify the solution is physically valid
        assert!(result.q_dot != HeatTransferRate::None);

        // UA should match target within tolerance
        assert_relative_eq!(result.ua.get::<kilowatt_per_kelvin>(), 2.0, epsilon = 1e-12);

        // Verify no temperature crossover in the final solution
        assert!(
            result
                .min_delta_t
                .value
                .get::<uom::si::temperature_interval::kelvin>()
                > 0.0
        );

        // Top stream should cool, bottom stream should heat
        assert!(result.top[4].temperature < known.inlets.top.temperature);
        assert!(result.bottom[0].temperature > known.inlets.bottom.temperature);
    }
}
