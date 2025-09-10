use std::ops::Mul;

use twine_core::TimeDerivative;
use twine_thermo::units::TemperatureDifference;
use uom::si::f64::{
    Power, TemperatureInterval, ThermalConductance, ThermodynamicTemperature, VolumeRate,
};

use super::{Adjacent, InverseHeatCapacity, InverseVolume};

/// Returns `dT/dt` due to fluid flows.
///
/// Mass is conserved under incompressible assumptions by matching all
/// inflows with an assumed equal outflow at the current node temperature.
/// Therefore, the temperature derivative due to fluid flow is only a
/// function of normalized inbound flows and is calculated according to:
/// ```text
/// dT/dt = Σ[V_dot_in · (T_in − T_node)] / V_node
/// ```
pub(super) fn derivative_from_fluid_flows(
    t_node: ThermodynamicTemperature,
    inv_vol: InverseVolume,
    inflows: impl IntoIterator<Item = (VolumeRate, ThermodynamicTemperature)>,
) -> TimeDerivative<ThermodynamicTemperature> {
    inflows
        .into_iter()
        .map(|(v_dot_in, t_in)| v_dot_in * t_in.minus(t_node))
        .sum::<TemperatureFlow>()
        * inv_vol
}

/// Returns `dT/dt` due to auxiliary heat sources.
///
/// The temperature derivative is calculated according to:
/// ```text
/// dT/dt = Σ[Q_dot_aux] / (ρ · c · V)
/// ```
pub(super) fn derivative_from_heat_flows(
    inv_heat_capacity: InverseHeatCapacity,
    heat_flows: impl IntoIterator<Item = Power>,
) -> TimeDerivative<ThermodynamicTemperature> {
    heat_flows.into_iter().sum::<Power>() * inv_heat_capacity
}

/// Returns `dT/dt` due to conduction.
///
/// Conduction through the bottom, side, and top of the node are considered,
/// with the net conduction loss (or gain) calculated according to:
/// ```text
/// Q_dot_cond = UA_bottom · (T_bottom − T_node)
///            + UA_side · (T_side − T_node)
///            + UA_top · (T_top − T_node)
/// ```
/// The temperature derivative is then calculated according to:
/// ```text
/// dT/dt = Q_dot_cond / (ρ · c · V)
/// ```
pub(super) fn deriv_from_conduction(
    t_node: ThermodynamicTemperature,
    t_adj: Adjacent<ThermodynamicTemperature>,
    ua: Adjacent<ThermalConductance>,
    inv_heat_capacity: InverseHeatCapacity,
) -> TimeDerivative<ThermodynamicTemperature> {
    let q_dot_cond = ua.bottom * t_adj.bottom.minus(t_node)
        + ua.side * t_adj.side.minus(t_node)
        + ua.top * t_adj.top.minus(t_node);

    q_dot_cond * inv_heat_capacity
}

type TemperatureFlow = <VolumeRate as Mul<TemperatureInterval>>::Output;
