use std::ops::{Div, Mul};

use twine_core::TimeDerivative;
use twine_thermo::units::TemperatureDifference;
use uom::si::f64::{
    HeatCapacity, Power, Ratio, TemperatureInterval, ThermalConductance, ThermodynamicTemperature,
    Volume, VolumeRate,
};

/// A single, fully-mixed layer of the stratified tank.
///
/// Stores inverse volume (`1 / V`), inverse thermal capacity (`1 / (ρ · c · V)`),
/// and conduction `UA` terms to its surrounding temperatures (bottom/side/top).
#[derive(Debug, Clone, Copy)]
pub(super) struct Node {
    inv_volume: InverseVolume,
    inv_heat_capacity: InverseHeatCapacity,
    ua_bottom: ThermalConductance,
    ua_side: ThermalConductance,
    ua_top: ThermalConductance,
}

impl Node {
    pub(super) fn new(
        volume: Volume,
        heat_capacity: HeatCapacity,
        ua_bottom: ThermalConductance,
        ua_side: ThermalConductance,
        ua_top: ThermalConductance,
    ) -> Self {
        Self {
            inv_volume: volume.recip(),
            inv_heat_capacity: heat_capacity.recip(),
            ua_bottom,
            ua_side,
            ua_top,
        }
    }

    /// Returns `dT/dt` due to fluid flows.
    ///
    /// Mass is conserved in the node by matching all inflows with an assumed
    /// equal outflow at the node temperature.
    /// Therefore, the temperature derivative due to fluid flow is only a
    /// function of normalized inbound flows and is calculated according to:
    /// ```text
    /// dT/dt = Σ[V_dot · (T_in − T_node)] / V
    /// ```
    pub(super) fn derivative_from_fluid_flows(
        &self,
        t_node: ThermodynamicTemperature,
        inflows: impl IntoIterator<Item = (VolumeRate, ThermodynamicTemperature)>,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        inflows
            .into_iter()
            .map(|(v_dot_in, t_in)| v_dot_in * t_in.minus(t_node))
            .sum::<TemperatureFlow>()
            * self.inv_volume
    }

    /// Returns `dT/dt` due to auxiliary heat sources.
    ///
    /// The temperature derivative is calculated according to:
    /// ```text
    /// dT/dt = Σ[Q_dot_aux] / (ρ · c · V)
    /// ```
    pub(super) fn derivative_from_heat_flows(
        &self,
        heat_flows: impl IntoIterator<Item = Power>,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        heat_flows.into_iter().sum::<Power>() * self.inv_heat_capacity
    }

    /// Returns `dT/dt` due to conduction.
    ///
    /// Conduction through the bottom, side, and top of the node are considered,
    /// with the net conduction loss (or gain) calculated according to:
    /// ```text
    /// Q_dot_cond = UA_bottom · (T_bottom − T_center)
    ///            + UA_side · (T_side − T_center)
    ///            + UA_top · (T_top − T_center)
    /// ```
    /// The temperature derivative is then calculated according to:
    /// ```text
    /// dT/dt = Q_dot_cond / (ρ · c · V)
    /// ```
    pub(super) fn derivative_from_conduction(
        &self,
        t: NodeTemperatures,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let q_dot_net = self.ua_bottom * t.bottom.minus(t.center)
            + self.ua_side * t.side.minus(t.center)
            + self.ua_top * t.top.minus(t.center);

        q_dot_net * self.inv_heat_capacity
    }
}

/// Temperatures used for conduction to/from this node.
///
/// Values for `bottom` and `top` may be ambient (if at tank boundary) or the
/// temperatures of adjacent nodes (for internal layers).
#[derive(Debug, Clone, Copy)]
pub(super) struct NodeTemperatures {
    /// The node's own temperature (center point).
    center: ThermodynamicTemperature,
    /// Temperature adjacent to the bottom face.
    bottom: ThermodynamicTemperature,
    /// Temperature adjacent to the side surface.
    side: ThermodynamicTemperature,
    /// Temperature adjacent to the top face.
    top: ThermodynamicTemperature,
}

type InverseVolume = <Ratio as Div<Volume>>::Output;
type InverseHeatCapacity = <Ratio as Div<HeatCapacity>>::Output;
type TemperatureFlow = <VolumeRate as Mul<TemperatureInterval>>::Output;

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{
            HeatCapacity, Power, ThermalConductance, ThermodynamicTemperature, Volume, VolumeRate,
        },
        heat_capacity::joule_per_kelvin,
        power::watt,
        thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin},
        volume::cubic_meter as m3,
        volume_rate::cubic_meter_per_second,
    };

    /// Implement a few methods on `Node` that are useful for testing.
    impl Node {
        pub fn test() -> Self {
            Node::new(
                Volume::new::<m3>(1.0),
                HeatCapacity::new::<joule_per_kelvin>(1.0),
                ThermalConductance::new::<watt_per_kelvin>(0.0),
                ThermalConductance::new::<watt_per_kelvin>(0.0),
                ThermalConductance::new::<watt_per_kelvin>(0.0),
            )
        }

        pub fn with_volume(self, v: f64) -> Self {
            Self {
                inv_volume: Volume::new::<m3>(v).recip(),
                ..self
            }
        }

        pub fn with_heat_capacity(self, c: f64) -> Self {
            Self {
                inv_heat_capacity: HeatCapacity::new::<joule_per_kelvin>(c).recip(),
                ..self
            }
        }

        pub fn with_ua_bottom(self, ua: f64) -> Self {
            Self {
                ua_bottom: ThermalConductance::new::<watt_per_kelvin>(ua),
                ..self
            }
        }
        pub fn with_ua_side(self, ua: f64) -> Self {
            Self {
                ua_side: ThermalConductance::new::<watt_per_kelvin>(ua),
                ..self
            }
        }
        pub fn with_ua_top(self, ua: f64) -> Self {
            Self {
                ua_top: ThermalConductance::new::<watt_per_kelvin>(ua),
                ..self
            }
        }
    }

    /// Creates a temperature in K.
    fn t(t: f64) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(t)
    }

    /// Creates a flow rate in m³/s.
    fn v_dot(v: f64) -> VolumeRate {
        VolumeRate::new::<cubic_meter_per_second>(v)
    }

    #[test]
    fn nothing_changes_at_equilibrium() {
        let node = Node::test();
        let t_node = t(300.0);

        let flow_deriv = node.derivative_from_fluid_flows(
            t_node,
            iter::empty::<(VolumeRate, ThermodynamicTemperature)>(),
        );
        assert_relative_eq!(flow_deriv.value, 0.0);

        let aux_deriv = node.derivative_from_heat_flows(iter::empty::<Power>());
        assert_relative_eq!(aux_deriv.value, 0.0);

        let cond_deriv = node.derivative_from_conduction(NodeTemperatures {
            center: t_node,
            bottom: t_node,
            side: t_node,
            top: t_node,
        });
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    // ---------- Fluid flows ----------

    #[test]
    fn fluid_flows_basic_heating() {
        // V = 1 m³, ΔT = +10 K, V_dot = 0.1 m³/s -> dT/dt = 1 K/s
        let node = Node::test();
        let t_node = t(300.0);

        let flow_deriv = node.derivative_from_fluid_flows(t_node, [(v_dot(0.1), t(310.0))]);
        assert_relative_eq!(flow_deriv.value, 1.0);
    }

    #[test]
    fn fluid_flows_cancellation() {
        let node = Node::test().with_volume(2.0);
        let t_node = t(350.0);

        // Equal/opposite ΔT with equal flows.
        let flow_deriv = node.derivative_from_fluid_flows(
            t_node,
            [
                (v_dot(0.2), t(380.0)), // +30 K
                (v_dot(0.2), t(320.0)), // -30 K
            ],
        );
        assert_relative_eq!(flow_deriv.value, 0.0);

        // Unequal ΔT with corresponding unequal flows (weighted cancel).
        let flow_deriv = node.derivative_from_fluid_flows(
            t_node,
            [
                (v_dot(0.3), t(360.0)),  // +10 K (twice the rate)
                (v_dot(0.15), t(330.0)), // -20 K
            ],
        );
        assert_relative_eq!(flow_deriv.value, 0.0);
    }

    #[test]
    fn fluid_flows_mixed_temperature_units_are_equal() {
        // 25 °C vs 77 °F are equal temps => ΔT = 0 no matter the flow
        let node = Node::test();
        let t_node = ThermodynamicTemperature::new::<degree_celsius>(25.0);
        let t_in = ThermodynamicTemperature::new::<degree_fahrenheit>(77.0);

        let flow_deriv = node.derivative_from_fluid_flows(t_node, [(v_dot(10.0), t_in)]);
        assert_relative_eq!(flow_deriv.value, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn fluid_flows_multiple_terms_match_hand_calc() {
        // V = 1.5 m³, sum(V_dot * ΔT) / V
        let node = Node::test().with_volume(1.5);
        let t_node = t(300.0);
        let inflows = [
            (v_dot(0.05), t(315.0)), // ΔT = +15
            (v_dot(0.02), t(290.0)), // ΔT = -10
            (v_dot(0.01), t(305.0)), // ΔT = +5
        ];
        let expected = (0.05 * 15.0 + 0.02 * -10.0 + 0.01 * 5.0) / 1.5;

        let flow_deriv = node.derivative_from_fluid_flows(t_node, inflows);
        assert_relative_eq!(flow_deriv.value, expected);
    }

    // ---------- Auxiliary heat ----------

    #[test]
    fn aux_heat_sums_and_scales() {
        // ΣQ_dot = 1200 W, C = 600 J/K => dT/dt = 2 K/s
        let node = Node::test().with_heat_capacity(600.0);
        let aux_deriv = node.derivative_from_heat_flows([
            Power::new::<watt>(500.0),
            Power::new::<watt>(700.0),
            Power::new::<watt>(-100.0),
            Power::new::<watt>(50.0),
            Power::new::<watt>(50.0),
        ]);
        assert_relative_eq!(aux_deriv.value, 2.0);
    }

    // ---------- Conduction ----------

    #[test]
    fn conduction_zero_when_equal_surroundings() {
        let node = Node::test()
            .with_ua_bottom(10.0)
            .with_ua_side(10.0)
            .with_ua_top(10.0);
        let t_node = t(300.0);

        let cond_deriv = node.derivative_from_conduction(NodeTemperatures {
            center: t_node,
            bottom: t_node,
            side: t_node,
            top: t_node,
        });
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    #[test]
    fn conduction_sign_and_magnitude_bottom_only() {
        // UA_bottom=10 W/K; ΔT=+5 K => Q_dot=50 W; C=25 J/K => dT/dt=2 K/s
        let node = Node::test().with_heat_capacity(25.0).with_ua_bottom(10.0);
        let t_node = t(300.0);

        let cond_deriv = node.derivative_from_conduction(NodeTemperatures {
            center: t_node,
            bottom: t(305.0),
            side: t_node,
            top: t_node,
        });
        assert_relative_eq!(cond_deriv.value, 2.0);
    }

    #[test]
    fn conduction_cancel_bottom_vs_top() {
        // UA_b=4, UA_t=6; ΔT_b=+3 K, ΔT_t=-2 K => Q̇ = 4*3 + 6*(-2) = 0 ⇒ dT/dt = 0
        let node = Node::test().with_ua_bottom(4.0).with_ua_top(6.0);
        let t_node = t(275.0);

        let cond_deriv = node.derivative_from_conduction(NodeTemperatures {
            center: t_node,
            bottom: t(278.0),
            side: t_node,
            top: t(273.0),
        });
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    #[test]
    fn conduction_superposition_all_faces() {
        // UA_b=5, UA_s=7, UA_t=9; ΔT_b=+1, ΔT_s=-2, ΔT_t=+3:
        // Q_dot= 5*1 + 7*(-2) + 9*3 = 5 - 14 + 27 = 18 W;  C = 6 J/K => dT/dt = 3 K/s
        let node = Node::test()
            .with_heat_capacity(6.0)
            .with_ua_bottom(5.0)
            .with_ua_side(7.0)
            .with_ua_top(9.0);

        let cond_deriv = node.derivative_from_conduction(NodeTemperatures {
            center: t(300.0),
            bottom: t(301.0),
            side: t(298.0),
            top: t(303.0),
        });
        assert_relative_eq!(cond_deriv.value, 3.0);
    }
}
