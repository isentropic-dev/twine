//! Performance metrics for discretized heat exchangers.

use twine_thermo::units::TemperatureDifference;
use uom::{
    ConstZero,
    si::f64::{MassRate, TemperatureInterval, ThermalConductance},
    si::temperature_interval::kelvin as delta_kelvin,
};

use crate::thermal::hx::{
    CapacitanceRate, Stream, StreamInlet, discretized::HeatTransferRate, functional,
};

use super::{
    discretize::Nodes, error::SolveError, results::MinDeltaT, traits::DiscretizedArrangement,
};

/// Computes total UA using a segment-by-segment effectiveness-NTU analysis.
pub(super) fn compute_ua<Arrangement, TopFluid, BottomFluid, const N: usize>(
    arrangement: &Arrangement,
    m_dot_top: MassRate,
    m_dot_bottom: MassRate,
    q_dot: HeatTransferRate,
    nodes: &Nodes<TopFluid, BottomFluid, N>,
) -> Result<ThermalConductance, SolveError>
where
    Arrangement: DiscretizedArrangement,
{
    if q_dot == HeatTransferRate::None {
        return Ok(ThermalConductance::ZERO);
    }

    let bottom_outlet_index = Arrangement::bottom_select(N - 1, 0);

    let mut ua_total = ThermalConductance::ZERO;

    for i in 0..(N - 1) {
        let top_in = &nodes.top[i];
        let top_out = &nodes.top[i + 1];
        let h_top_in = nodes.top_enthalpies[i];
        let h_top_out = nodes.top_enthalpies[i + 1];

        let (bottom_in, bottom_out) = Arrangement::bottom_select(
            (&nodes.bottom[i], &nodes.bottom[i + 1]),
            (&nodes.bottom[i + 1], &nodes.bottom[i]),
        );

        let (h_bottom_in, h_bottom_out) = Arrangement::bottom_select(
            (nodes.bottom_enthalpies[i], nodes.bottom_enthalpies[i + 1]),
            (nodes.bottom_enthalpies[i + 1], nodes.bottom_enthalpies[i]),
        );

        let segment_delta_t_top_to_bottom = top_in.temperature.minus(bottom_in.temperature);
        let segment_delta_t_bottom_to_top = bottom_in.temperature.minus(top_in.temperature);
        let segment_delta_t_hot_cold = match q_dot {
            HeatTransferRate::BottomToTop(_) => segment_delta_t_bottom_to_top,
            HeatTransferRate::TopToBottom(_) | HeatTransferRate::None => {
                segment_delta_t_top_to_bottom
            }
        };

        let top_delta_t = top_out.temperature.minus(top_in.temperature);
        let top_delta_h = h_top_out - h_top_in;
        let c_dot_top = m_dot_top * top_delta_h / top_delta_t;
        let c_dot_top = CapacitanceRate::from_quantity(c_dot_top).map_err(|_| {
            segment_violation_error(
                nodes,
                q_dot,
                segment_delta_t_top_to_bottom,
                i,
                bottom_outlet_index,
            )
        })?;

        let bottom_delta_t = bottom_out.temperature.minus(bottom_in.temperature);
        let bottom_delta_h = h_bottom_out - h_bottom_in;
        let c_dot_bottom = m_dot_bottom * bottom_delta_h / bottom_delta_t;
        let c_dot_bottom = CapacitanceRate::from_quantity(c_dot_bottom).map_err(|_| {
            segment_violation_error(
                nodes,
                q_dot,
                segment_delta_t_bottom_to_top,
                i,
                bottom_outlet_index,
            )
        })?;

        let functional::KnownConditionsResult { ua, .. } = functional::known_conditions_and_inlets(
            arrangement,
            (
                StreamInlet::new(c_dot_top, top_in.temperature),
                Stream::new_from_outlet_temperature(
                    c_dot_bottom,
                    bottom_in.temperature,
                    bottom_out.temperature,
                ),
            ),
        )
        .map_err(|_| {
            segment_violation_error(
                nodes,
                q_dot,
                segment_delta_t_hot_cold,
                i,
                bottom_outlet_index,
            )
        })?;

        ua_total += ua;
    }

    Ok(ua_total)
}

/// Computes the minimum hot-to-cold temperature difference and its node index.
pub(super) fn compute_min_delta_t<Arrangement, TopFluid, BottomFluid, const N: usize>(
    nodes: &Nodes<TopFluid, BottomFluid, N>,
) -> MinDeltaT
where
    Arrangement: DiscretizedArrangement,
{
    if N == 0 {
        return MinDeltaT {
            value: TemperatureInterval::ZERO,
            node: 0,
        };
    }

    let top_inlet = nodes.top[0].temperature;
    let bottom_inlet_index = Arrangement::bottom_select(0, N - 1);
    let bottom_inlet = nodes.bottom[bottom_inlet_index].temperature;
    let top_is_hot = top_inlet >= bottom_inlet;

    let mut min_delta_t = TemperatureInterval::new::<delta_kelvin>(f64::INFINITY);
    let mut min_node = 0;

    for i in 0..N {
        let delta_t = if top_is_hot {
            nodes.top[i].temperature.minus(nodes.bottom[i].temperature)
        } else {
            nodes.bottom[i].temperature.minus(nodes.top[i].temperature)
        };

        if delta_t < min_delta_t {
            min_delta_t = delta_t;
            min_node = i;
        }
    }

    MinDeltaT {
        value: min_delta_t,
        node: min_node,
    }
}

/// Creates a second law violation error for a segment with invalid capacitance rate.
fn segment_violation_error<TopFluid, BottomFluid, const N: usize>(
    nodes: &Nodes<TopFluid, BottomFluid, N>,
    q_dot: HeatTransferRate,
    segment_delta_t: TemperatureInterval,
    segment_index: usize,
    bottom_outlet_index: usize,
) -> SolveError {
    // Safety: N >= 2 is enforced at API entry points via const assertion
    SolveError::SecondLawViolation {
        top_outlet_temp: Some(nodes.top[N - 1].temperature),
        bottom_outlet_temp: Some(nodes.bottom[bottom_outlet_index].temperature),
        q_dot: q_dot.signed_top_to_bottom(),
        min_delta_t: segment_delta_t,
        violation_node: Some(segment_index),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{f64::MassRate, mass_rate::kilogram_per_second};

    use crate::thermal::hx::{
        arrangement::CounterFlow,
        discretized::{
            Given, HeatTransferRate, Inlets, Known, MassFlows, PressureDrops,
            resolve::Resolved,
            test_support::{TestThermoModel, state},
        },
    };

    #[test]
    fn ua_is_zero_for_no_heat_transfer() {
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
            Given::HeatTransferRate(HeatTransferRate::None),
            &model,
            &model,
        )
        .expect("resolution should succeed");

        let nodes = Nodes::<_, _, 2>::new::<CounterFlow>(&resolved, &model, &model)
            .expect("discretization should succeed");

        let ua = compute_ua(
            &CounterFlow,
            resolved.top.m_dot,
            resolved.bottom.m_dot,
            resolved.q_dot,
            &nodes,
        )
        .expect("metrics should succeed");

        assert_eq!(ua, ThermalConductance::ZERO);
    }
}
