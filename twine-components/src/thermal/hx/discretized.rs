//! Discretized counterflow and parallel-flow heat exchanger modeling.
//!
//! A discretized heat exchanger divides the flow into a linear series of
//! constant-property sub-exchangers so thermodynamic properties can vary
//! along a linear array of nodes, supporting real-fluid behavior.

mod discretize;
mod error;
mod heat_transfer_rate;
mod input;
mod metrics;
mod resolve;
mod results;
#[cfg(test)]
mod test_support;
mod traits;

pub use error::SolveError;
pub use heat_transfer_rate::HeatTransferRate;
pub use input::{Given, Inlets, Known, MassFlows, PressureDrops};
pub use results::{MinDeltaT, Results};

use std::marker::PhantomData;

use discretize::Nodes;
use metrics::{compute_min_delta_t, compute_ua};
use resolve::Resolved;
use traits::{DiscretizedArrangement, DiscretizedHxThermoModel};
use uom::{ConstZero, si::f64::TemperatureInterval};

/// Entry point for solving a discretized heat exchanger.
///
/// The arrangement and node count are fixed by generics.
/// This type also provides the natural home for higher-level solve helpers,
/// such as iterating on UA to match target outlet states.
///
/// # Minimum Node Count
///
/// The node count `N` must be at least 2 (inlet and outlet).
/// This constraint is enforced at compile time via const assertions.
///
/// ```compile_fail
/// # use twine_components::thermal::hx::{arrangement::ParallelFlow, discretized::DiscretizedHx};
/// // This will fail to compile: N must be >= 2
/// let _ = DiscretizedHx::<ParallelFlow, 1>::solve(todo!(), todo!(), todo!(), todo!());
/// ```
///
/// # Example
///
/// ```rust
/// use twine_components::thermal::hx::{
///     arrangement::CounterFlow,
///     discretized::{DiscretizedHx, Given, HeatTransferRate, Inlets, Known, MassFlows, PressureDrops},
/// };
/// use twine_thermo::{fluid::CarbonDioxide, model::perfect_gas::PerfectGas};
/// use uom::si::f64::{MassRate, Pressure};
/// use uom::si::{mass_rate::kilogram_per_second, pressure::pascal};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let thermo = PerfectGas::<CarbonDioxide>::new()?;
/// let inlet = thermo.reference_state(CarbonDioxide);
///
/// let known = Known {
///     inlets: Inlets {
///         top: inlet,
///         bottom: inlet,
///     },
///     m_dot: MassFlows::new(
///         MassRate::new::<kilogram_per_second>(1.0),
///         MassRate::new::<kilogram_per_second>(1.0),
///     )?,
///     dp: PressureDrops::zero(),
/// };
///
/// let given = Given::HeatTransferRate(HeatTransferRate::None);
///
/// let _result = DiscretizedHx::<CounterFlow, 20>::solve(&known, given, &thermo, &thermo)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DiscretizedHx<Arrangement, const N: usize> {
    _arrangement: PhantomData<Arrangement>,
}

impl<Arrangement, const N: usize> DiscretizedHx<Arrangement, N> {
    /// Solve a discretized heat exchanger with a fixed arrangement and node count.
    ///
    /// # Errors
    ///
    /// Returns a [`SolveError`] on non-physical results or thermodynamic model failures.
    pub fn solve<TopFluid, BottomFluid>(
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
        ensure_second_law::<Arrangement, N, _, _>(&resolved, &nodes, min_delta_t)?;

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

    /// Solve a discretized heat exchanger when both streams share the same thermo model.
    ///
    /// This is a convenience wrapper around [`DiscretizedHx::solve`].
    ///
    /// # Errors
    ///
    /// Returns a [`SolveError`] on non-physical results or thermodynamic model failures.
    pub fn solve_same<Fluid, Model>(
        known: &Known<Fluid, Fluid>,
        given: Given,
        thermo: &Model,
    ) -> Result<Results<Fluid, Fluid, N>, SolveError>
    where
        Arrangement: DiscretizedArrangement + Default,
        Fluid: Clone,
        Model: DiscretizedHxThermoModel<Fluid>,
    {
        Self::solve(known, given, thermo, thermo)
    }
}

/// Validate second-law constraints for the resolved solution.
fn ensure_second_law<Arrangement, const N: usize, TopFluid, BottomFluid>(
    resolved: &Resolved<TopFluid, BottomFluid>,
    nodes: &Nodes<TopFluid, BottomFluid, N>,
    min_delta_t: MinDeltaT,
) -> Result<(), SolveError>
where
    Arrangement: DiscretizedArrangement,
{
    if resolved.q_dot == HeatTransferRate::None {
        return Ok(());
    }

    let top_inlet = nodes.top[0].temperature;
    let bottom_inlet = nodes.bottom[0].temperature;
    let top_is_hot = top_inlet >= bottom_inlet;

    let direction_mismatch = match resolved.q_dot {
        HeatTransferRate::TopToBottom(_) => !top_is_hot,
        HeatTransferRate::BottomToTop(_) => top_is_hot,
        HeatTransferRate::None => false,
    };

    if direction_mismatch || min_delta_t.value <= TemperatureInterval::ZERO {
        let bottom_outlet_index = Arrangement::bottom_select(N - 1, 0);
        return Err(SolveError::SecondLawViolation {
            top_outlet_temp: Some(nodes.top[N - 1].temperature),
            bottom_outlet_temp: Some(nodes.bottom[bottom_outlet_index].temperature),
            q_dot: resolved.q_dot.signed_top_to_bottom(),
            min_delta_t: min_delta_t.value,
            violation_node: Some(min_delta_t.node),
        });
    }

    Ok(())
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
        functional,
    };

    use test_support::{TestThermoModel, state};

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

        let result = DiscretizedHx::<CounterFlow, 5>::solve(
            &known,
            Given::HeatTransferRate(q_dot),
            &model,
            &model,
        );

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
        let result = DiscretizedHx::<CounterFlow, 5>::solve(
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
        let result = DiscretizedHx::<CounterFlow, 5>::solve_same(
            &known,
            Given::HeatTransferRate(q_dot),
            &model,
        )
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
        let result = DiscretizedHx::<ParallelFlow, 5>::solve_same(
            &known,
            Given::HeatTransferRate(q_dot),
            &model,
        )
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
