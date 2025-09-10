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
pub(super) fn derivative_from_conduction(
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{HeatCapacity, Volume},
        heat_capacity::joule_per_kelvin,
        power::watt,
        thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::kelvin,
        volume::cubic_meter,
        volume_rate::cubic_meter_per_second,
    };

    /// Creates a temperature in K.
    fn t(value: f64) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(value)
    }

    /// Creates a flow rate in m3/s.
    fn v_dot(value: f64) -> VolumeRate {
        VolumeRate::new::<cubic_meter_per_second>(value)
    }

    /// Inverts a volume in m3.
    fn inv_vol(value: f64) -> InverseVolume {
        Volume::new::<cubic_meter>(value).recip()
    }

    /// Inverts a heat capacity in J/K.
    fn inv_c(value: f64) -> InverseHeatCapacity {
        HeatCapacity::new::<joule_per_kelvin>(value).recip()
    }

    #[test]
    fn nothing_changes_at_equilibrium() {
        let t_node = t(300.0);

        let flows = iter::empty::<(VolumeRate, ThermodynamicTemperature)>();
        let flow_deriv = derivative_from_fluid_flows(t_node, inv_vol(1.0), flows);
        assert_relative_eq!(flow_deriv.value, 0.0);

        let aux_heat = iter::empty::<Power>();
        let aux_deriv = derivative_from_heat_flows(inv_c(1.0), aux_heat);
        assert_relative_eq!(aux_deriv.value, 0.0);

        let ua = ThermalConductance::new::<watt_per_kelvin>(10.0);
        let cond_deriv = derivative_from_conduction(
            t_node,
            Adjacent {
                bottom: t_node,
                side: t_node,
                top: t_node,
            },
            Adjacent {
                bottom: ua,
                side: ua,
                top: ua,
            },
            inv_c(1.0),
        );
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    // ---------- Fluid flows ----------

    #[test]
    fn fluid_flows_basic_heating() {
        // V = 1 m3, ΔT = +10 K, V_dot = 0.1 m3/s -> dT/dt = 1 K/s
        let t_node = t(300.0);
        let flows = [(v_dot(0.1), t(310.0))];

        let flow_deriv = derivative_from_fluid_flows(t_node, inv_vol(1.0), flows);
        assert_relative_eq!(flow_deriv.value, 1.0);
    }

    #[test]
    fn fluid_flows_cancellation() {
        let t_node = t(350.0);

        // Equal/opposite ΔT with equal flows.
        let flow_deriv = derivative_from_fluid_flows(
            t_node,
            inv_vol(1.0),
            [
                (v_dot(0.2), t(380.0)), // +30 K
                (v_dot(0.2), t(320.0)), // -30 K
            ],
        );
        assert_relative_eq!(flow_deriv.value, 0.0);

        // Unequal ΔT with corresponding unequal flows (weighted cancel).
        let flow_deriv = derivative_from_fluid_flows(
            t_node,
            inv_vol(1.0),
            [
                (v_dot(0.3), t(360.0)),  // +10 K (twice the rate)
                (v_dot(0.15), t(330.0)), // -20 K
            ],
        );
        assert_relative_eq!(flow_deriv.value, 0.0);
    }

    #[test]
    fn fluid_flows_multiple_terms_match_hand_calc() {
        // V = 1.5 m3, T = 300 K, sum(V_dot * ΔT) / V
        let t_node = t(300.0);
        let inflows = [
            (v_dot(0.05), t(315.0)), // ΔT = +15
            (v_dot(0.02), t(290.0)), // ΔT = -10
            (v_dot(0.01), t(305.0)), // ΔT = +5
        ];
        let expected = (0.05 * 15.0 + 0.02 * -10.0 + 0.01 * 5.0) / 1.5;

        let flow_deriv = derivative_from_fluid_flows(t_node, inv_vol(1.5), inflows);
        assert_relative_eq!(flow_deriv.value, expected);
    }

    // ---------- Auxiliary heat ----------

    #[test]
    fn aux_heat_sums_and_scales() {
        // ΣQ_dot = 1.2 kW, C = 4 kJ/K => dT/dt = 0.3 K/s
        let aux_deriv = derivative_from_heat_flows(
            inv_c(4000.0),
            [
                Power::new::<watt>(500.0),
                Power::new::<watt>(700.0),
                Power::new::<watt>(-100.0),
                Power::new::<watt>(50.0),
                Power::new::<watt>(50.0),
            ],
        );
        assert_relative_eq!(aux_deriv.value, 0.3);
    }

    // ---------- Conduction ----------

    #[test]
    fn conduction_sign_and_magnitude_bottom_only() {
        // UA_bottom=10 W/K; ΔT=+5 K => Q_dot=50 W; C=25 J/K => dT/dt=2 K/s
        let t_node = t(300.0);
        let t_adj = Adjacent {
            bottom: t(305.0),
            side: t_node,
            top: t_node,
        };
        let ua = Adjacent {
            bottom: ThermalConductance::new::<watt_per_kelvin>(10.0),
            side: ThermalConductance::default(),
            top: ThermalConductance::default(),
        };

        let cond_deriv = derivative_from_conduction(t_node, t_adj, ua, inv_c(25.0));
        assert_relative_eq!(cond_deriv.value, 2.0);
    }

    #[test]
    fn conduction_cancel_bottom_vs_top() {
        // UA_b=4, UA_t=6; ΔT_b=+3 K, ΔT_t=-2 K => Q̇ = 4*3 + 6*(-2) = 0 ⇒ dT/dt = 0
        let t_node = t(275.0);
        let t_adj = Adjacent {
            bottom: t(278.0),
            side: t_node,
            top: t(273.0),
        };
        let ua = Adjacent {
            bottom: ThermalConductance::new::<watt_per_kelvin>(4.0),
            side: ThermalConductance::default(),
            top: ThermalConductance::new::<watt_per_kelvin>(6.0),
        };

        let cond_deriv = derivative_from_conduction(t_node, t_adj, ua, inv_c(1.0));
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    #[test]
    fn conduction_superposition_all_faces() {
        // UA_b=5, UA_s=7, UA_t=9; ΔT_b=+1, ΔT_s=-2, ΔT_t=+3:
        // Q_dot= 5*1 + 7*(-2) + 9*3 = 5 - 14 + 27 = 18 W;  C = 6 J/K => dT/dt = 3 K/s
        let t_node = t(400.0);
        let t_adj = Adjacent {
            bottom: t(401.0),
            side: t(398.0),
            top: t(403.0),
        };
        let ua = Adjacent {
            bottom: ThermalConductance::new::<watt_per_kelvin>(5.0),
            side: ThermalConductance::new::<watt_per_kelvin>(7.0),
            top: ThermalConductance::new::<watt_per_kelvin>(9.0),
        };

        let cond_deriv = derivative_from_conduction(t_node, t_adj, ua, inv_c(6.0));
        assert_relative_eq!(cond_deriv.value, 3.0);
    }
}
