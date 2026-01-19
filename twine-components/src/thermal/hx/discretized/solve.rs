//! Core discretized heat exchanger solver.

mod error;
mod nodes;
mod resolved;

pub use error::SolveError;
pub(super) use nodes::Nodes;
pub(super) use resolved::Resolved;

use super::{
    Given, Known, Results,
    metrics::{compute_min_delta_t, compute_ua},
    traits::{DiscretizedArrangement, DiscretizedHxThermoModel},
};

/// Solves a discretized heat exchanger with the given constraints.
///
/// This function performs the core solve logic: resolution of boundary
/// conditions, discretization into nodes, validation of thermodynamic
/// constraints, and computation of performance metrics.
///
/// # Errors
///
/// Returns [`SolveError`] on non-physical results or thermodynamic model failures.
pub(super) fn solve<Arrangement, TopFluid, BottomFluid, const N: usize>(
    known: &Known<TopFluid, BottomFluid>,
    given: Given,
    thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
    thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
) -> Result<Results<TopFluid, BottomFluid, N>, SolveError>
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

    let resolved = Resolved::new(known, given, thermo_top, thermo_bottom)?;
    let nodes = Nodes::new::<Arrangement>(&resolved, thermo_top, thermo_bottom)?;

    let min_delta_t = compute_min_delta_t::<Arrangement, _, _, N>(&nodes);
    SolveError::check_second_law(&resolved, min_delta_t)?;

    let ua = compute_ua(
        &Arrangement::default(),
        resolved.top.m_dot,
        resolved.bottom.m_dot,
        resolved.q_dot,
        &nodes,
    )?;

    Ok(Results {
        top: nodes.top,
        bottom: nodes.bottom,
        q_dot: resolved.q_dot,
        ua,
        min_delta_t,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use twine_thermo::HeatFlow;
    use uom::si::{
        f64::{MassRate, Power, ThermodynamicTemperature},
        mass_rate::kilogram_per_second,
        power::kilowatt,
        thermal_conductance::kilowatt_per_kelvin,
        thermodynamic_temperature::kelvin,
    };

    use crate::thermal::hx::{
        CapacitanceRate, Stream, StreamInlet,
        arrangement::{CounterFlow, ParallelFlow},
        discretized::{
            HeatTransferRate, Inlets, MassFlows, PressureDrops,
            test_support::{TestThermoModel, state},
        },
        functional,
    };

    #[test]
    fn rejects_second_law_violation() {
        let model = TestThermoModel::new();

        let known = Known {
            inlets: Inlets {
                top: state(300.0),    // cold stream
                bottom: state(400.0), // hot stream
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(1.0),
                MassRate::new::<kilogram_per_second>(1.0),
            ),
            dp: PressureDrops::default(),
        };

        // Request heat flow from cold to hot, which isn't physically possible
        let q_dot = HeatTransferRate::TopToBottom(Power::new::<kilowatt>(10.0));

        let result =
            solve::<CounterFlow, _, _, 5>(&known, Given::HeatTransferRate(q_dot), &model, &model);

        match result {
            Err(SolveError::SecondLawViolation {
                q_dot, min_delta_t, ..
            }) => {
                // Verify error reports the requested (invalid) heat transfer
                assert_relative_eq!(q_dot.get::<kilowatt>(), 10.0);
                // Verify error reports a valid temperature difference
                let delta_t_kelvin = min_delta_t.get::<uom::si::temperature_interval::kelvin>();
                assert!(
                    delta_t_kelvin > 0.0 && delta_t_kelvin.is_finite(),
                    "min_delta_t should be positive and finite, got: {delta_t_kelvin}"
                );
            }
            other => panic!("Expected SecondLawViolation, got: {other:?}"),
        }
    }

    #[test]
    fn rejects_temperature_crossover() {
        let model = TestThermoModel::new();

        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(1.0),
                MassRate::new::<kilogram_per_second>(1.0),
            ),
            dp: PressureDrops::default(),
        };

        // Request excessive cooling of top stream that causes temperature crossover
        // Top: 400K → 200K (cools by 200K)
        // Bottom: 300K → 500K (heats by 200K, energy balance)
        // In counterflow, bottom outlet (500K) > top inlet (400K), causing crossover
        let result = solve::<CounterFlow, _, _, 5>(
            &known,
            Given::TopOutletTemp(ThermodynamicTemperature::new::<kelvin>(200.0)),
            &model,
            &model,
        );

        match result {
            Err(SolveError::SecondLawViolation {
                min_delta_t,
                violation_node,
                ..
            }) => {
                // Verify the error reports a negative or zero min_delta_t (temperature crossover)
                let delta_t_kelvin = min_delta_t.get::<uom::si::temperature_interval::kelvin>();
                assert!(
                    delta_t_kelvin <= 0.0,
                    "min_delta_t should be non-positive for temperature crossover, got: {delta_t_kelvin}"
                );
                // Verify a specific node location is reported
                assert!(
                    violation_node.is_some(),
                    "violation_node should be reported for temperature crossover"
                );
            }
            other => panic!("Expected SecondLawViolation, got: {other:?}"),
        }
    }

    #[test]
    fn counterflow_ua_matches_functional_solver() {
        let model = TestThermoModel::new();

        let m_dot_top = MassRate::new::<kilogram_per_second>(2.0);
        let m_dot_bottom = MassRate::new::<kilogram_per_second>(3.0);

        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(m_dot_top, m_dot_bottom),
            dp: PressureDrops::default(),
        };

        let q_dot = HeatTransferRate::TopToBottom(Power::new::<kilowatt>(60.0));

        // Solve with discretized solver (N=5 nodes)
        let result =
            solve::<CounterFlow, _, _, 5>(&known, Given::HeatTransferRate(q_dot), &model, &model)
                .expect("discretized solve should succeed");

        // Verify outlet temperatures match expected (energy balance)
        assert_relative_eq!(result.top[4].temperature.get::<kelvin>(), 370.0);
        assert_relative_eq!(result.bottom[0].temperature.get::<kelvin>(), 320.0);

        // Solve with functional solver (constant cp assumption)
        let functional_result = functional::known_conditions_and_inlets(
            &CounterFlow,
            (
                StreamInlet::new(
                    CapacitanceRate::from_quantity(m_dot_top * model.cp()).unwrap(),
                    known.inlets.top.temperature,
                ),
                Stream::new_from_heat_flow(
                    CapacitanceRate::from_quantity(m_dot_bottom * model.cp()).unwrap(),
                    known.inlets.bottom.temperature,
                    HeatFlow::outgoing(q_dot.magnitude()).unwrap(),
                ),
            ),
        )
        .expect("functional solve should succeed");

        // UA should match between discretized and functional solvers
        assert_relative_eq!(
            result.ua.get::<kilowatt_per_kelvin>(),
            functional_result.ua.get::<kilowatt_per_kelvin>(),
            epsilon = 1e-12,
        );
    }

    #[test]
    fn parallel_flow_ua_matches_functional_solver() {
        let model = TestThermoModel::new();

        let m_dot_top = MassRate::new::<kilogram_per_second>(2.0);
        let m_dot_bottom = MassRate::new::<kilogram_per_second>(3.0);

        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(m_dot_top, m_dot_bottom),
            dp: PressureDrops::default(),
        };

        let q_dot = HeatTransferRate::TopToBottom(Power::new::<kilowatt>(60.0));

        // Solve with discretized solver (N=5 nodes)
        let result =
            solve::<ParallelFlow, _, _, 5>(&known, Given::HeatTransferRate(q_dot), &model, &model)
                .expect("discretized solve should succeed");

        // Verify outlet temperatures match expected (energy balance)
        assert_relative_eq!(result.top[4].temperature.get::<kelvin>(), 370.0);
        assert_relative_eq!(result.bottom[4].temperature.get::<kelvin>(), 320.0);

        // Solve with functional solver (constant cp assumption)
        let functional_result = functional::known_conditions_and_inlets(
            &ParallelFlow,
            (
                StreamInlet::new(
                    CapacitanceRate::from_quantity(m_dot_top * model.cp()).unwrap(),
                    known.inlets.top.temperature,
                ),
                Stream::new_from_heat_flow(
                    CapacitanceRate::from_quantity(m_dot_bottom * model.cp()).unwrap(),
                    known.inlets.bottom.temperature,
                    HeatFlow::outgoing(q_dot.magnitude()).unwrap(),
                ),
            ),
        )
        .expect("functional solve should succeed");

        // UA should match between discretized and functional solvers
        assert_relative_eq!(
            result.ua.get::<kilowatt_per_kelvin>(),
            functional_result.ua.get::<kilowatt_per_kelvin>(),
        );
    }
}
