use std::ops::{Div, Mul};

use twine_core::TimeDerivative;
use twine_thermo::units::TemperatureDifference;
use uom::si::f64::{
    HeatCapacity, Mass, MassRate, Power, Ratio, SpecificHeatCapacity, TemperatureInterval,
    ThermalConductance, ThermodynamicTemperature,
};

use super::Adjacent;

/// Runtime thermal state of a node (temperature, mass, capacity).
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Layer {
    pub(super) temp: ThermodynamicTemperature,
    pub(super) mass: Mass,
    pub(super) inv_heat_capacity: InverseHeatCapacity,
}

impl Layer {
    /// Creates a new layer with the given temperature, mass, and specific heat capacity.
    pub(super) fn new(
        temp: ThermodynamicTemperature,
        mass: Mass,
        cp: SpecificHeatCapacity,
    ) -> Self {
        Self {
            temp,
            mass,
            inv_heat_capacity: (mass * cp).recip(),
        }
    }

    /// Returns `dT/dt` due to fluid flows.
    ///
    /// Mass is conserved under Boussinesq assumptions by matching all inflows with
    /// an assumed equal outflow at the layer temperature.
    /// Therefore, the temperature derivative due to fluid flow is only a
    /// function of normalized inbound flows and is calculated according to:
    /// ```text
    /// dT/dt = Σ[m_dot_in · (T_in − T_layer)] / m_layer
    /// ```
    pub(super) fn derivative_from_fluid_flows(
        &self,
        inflows: impl IntoIterator<Item = (MassRate, ThermodynamicTemperature)>,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        inflows
            .into_iter()
            .map(|(m_dot_in, t_in)| m_dot_in * t_in.minus(self.temp))
            .sum::<TemperatureFlow>()
            / self.mass
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
    /// Conduction through the bottom, side, and top of the layer are considered,
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
        ua: Adjacent<ThermalConductance>,
        t: Adjacent<ThermodynamicTemperature>,
    ) -> TimeDerivative<ThermodynamicTemperature> {
        let q_dot_net = ua.bottom * t.bottom.minus(self.temp)
            + ua.side * t.side.minus(self.temp)
            + ua.top * t.top.minus(self.temp);

        q_dot_net * self.inv_heat_capacity
    }

    /// Extracts the temperature from this layer, consuming self.
    pub(super) fn into_temperature(self) -> ThermodynamicTemperature {
        self.temp
    }
}

type InverseHeatCapacity = <Ratio as Div<HeatCapacity>>::Output;
type TemperatureFlow = <MassRate as Mul<TemperatureInterval>>::Output;

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter;

    use approx::assert_relative_eq;
    use uom::si::{
        mass::kilogram,
        mass_rate::kilogram_per_second,
        power::watt,
        specific_heat_capacity::{joule_per_kilogram_kelvin, kilojoule_per_kilogram_kelvin},
        thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin},
    };

    /// Creates a test layer at the given temperature.
    fn layer_at_temp(temp: ThermodynamicTemperature) -> Layer {
        Layer::new(
            temp,
            Mass::new::<kilogram>(1.0),
            SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.0),
        )
    }

    /// Creates a temperature in K.
    fn t(value: f64) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(value)
    }

    /// Creates a flow rate in kg/s.
    fn m_dot(value: f64) -> MassRate {
        MassRate::new::<kilogram_per_second>(value)
    }

    #[test]
    fn nothing_changes_at_equilibrium() {
        let layer = layer_at_temp(t(300.0));

        let flows = iter::empty::<(MassRate, ThermodynamicTemperature)>();
        let flow_deriv = layer.derivative_from_fluid_flows(flows);
        assert_relative_eq!(flow_deriv.value, 0.0);

        let aux_heat = iter::empty::<Power>();
        let aux_deriv = layer.derivative_from_heat_flows(aux_heat);
        assert_relative_eq!(aux_deriv.value, 0.0);

        let cond_deriv = layer.derivative_from_conduction(
            Adjacent::<ThermalConductance>::default(),
            Adjacent::from_value(t(300.0)),
        );
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    // ---------- Fluid flows ----------

    #[test]
    fn fluid_flows_basic_heating() {
        // M = 1 kg, ΔT = +10 K, m_dot = 0.1 kg/s -> dT/dt = 1 K/s
        let layer = layer_at_temp(t(300.0));
        let flows = [(m_dot(0.1), t(310.))];

        let flow_deriv = layer.derivative_from_fluid_flows(flows);
        assert_relative_eq!(flow_deriv.value, 1.0);
    }

    #[test]
    fn fluid_flows_cancellation() {
        let layer = layer_at_temp(t(350.0));

        // Equal/opposite ΔT with equal flows.
        let flow_deriv = layer.derivative_from_fluid_flows([
            (m_dot(0.2), t(380.0)), // +30 K
            (m_dot(0.2), t(320.0)), // -30 K
        ]);
        assert_relative_eq!(flow_deriv.value, 0.0);

        // Unequal ΔT with corresponding unequal flows (weighted cancel).
        let flow_deriv = layer.derivative_from_fluid_flows([
            (m_dot(0.3), t(360.0)),  // +10 K (twice the rate)
            (m_dot(0.15), t(330.0)), // -20 K
        ]);
        assert_relative_eq!(flow_deriv.value, 0.0);
    }

    #[test]
    fn fluid_flows_mixed_temperature_units_are_equal() {
        // 25 °C vs 77 °F are equal temps => ΔT = 0 no matter the flow
        let layer = layer_at_temp(ThermodynamicTemperature::new::<degree_celsius>(25.0));
        let t_in = ThermodynamicTemperature::new::<degree_fahrenheit>(77.0);

        let flow_deriv = layer.derivative_from_fluid_flows([(m_dot(10.0), t_in)]);
        assert_relative_eq!(flow_deriv.value, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn fluid_flows_multiple_terms_match_hand_calc() {
        // m = 1.5 kg, sum(m_dot * ΔT) / m
        let layer = Layer::new(
            t(300.0),
            Mass::new::<kilogram>(1.5),
            SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.0),
        );
        let inflows = [
            (m_dot(0.05), t(315.0)), // ΔT = +15
            (m_dot(0.02), t(290.0)), // ΔT = -10
            (m_dot(0.01), t(305.0)), // ΔT = +5
        ];
        let expected = (0.05 * 15.0 + 0.02 * -10.0 + 0.01 * 5.0) / 1.5;

        let flow_deriv = layer.derivative_from_fluid_flows(inflows);
        assert_relative_eq!(flow_deriv.value, expected);
    }

    // ---------- Auxiliary heat ----------

    #[test]
    fn aux_heat_sums_and_scales() {
        // ΣQ_dot = 1.2 kW, C = 4 kJ/K => dT/dt = 0.3 K/s
        let layer = layer_at_temp(t(320.0));
        let aux_deriv = layer.derivative_from_heat_flows([
            Power::new::<watt>(500.0),
            Power::new::<watt>(700.0),
            Power::new::<watt>(-100.0),
            Power::new::<watt>(50.0),
            Power::new::<watt>(50.0),
        ]);
        assert_relative_eq!(aux_deriv.value, 0.3);
    }

    // ---------- Conduction ----------

    #[test]
    fn conduction_zero_when_equal_surroundings() {
        let temp = t(300.0);
        let layer = layer_at_temp(temp);
        let cond_deriv = layer.derivative_from_conduction(
            Adjacent::from_value(ThermalConductance::new::<watt_per_kelvin>(10.0)),
            Adjacent::from_value(temp),
        );
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    #[test]
    fn conduction_sign_and_magnitude_bottom_only() {
        // UA_bottom=10 W/K; ΔT=+5 K => Q_dot=50 W; C=25 J/K => dT/dt=2 K/s
        let temp = t(300.0);
        let layer = Layer::new(
            temp,
            Mass::new::<kilogram>(1.0),
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(25.0),
        );
        let ua = Adjacent {
            bottom: ThermalConductance::new::<watt_per_kelvin>(10.0),
            side: ThermalConductance::default(),
            top: ThermalConductance::default(),
        };
        let adj_t = Adjacent {
            bottom: t(305.0),
            side: temp,
            top: temp,
        };

        let cond_deriv = layer.derivative_from_conduction(ua, adj_t);
        assert_relative_eq!(cond_deriv.value, 2.0);
    }

    #[test]
    fn conduction_cancel_bottom_vs_top() {
        // UA_b=4, UA_t=6; ΔT_b=+3 K, ΔT_t=-2 K => Q̇ = 4*3 + 6*(-2) = 0 ⇒ dT/dt = 0
        let layer = layer_at_temp(t(275.0));
        let ua = Adjacent {
            bottom: ThermalConductance::new::<watt_per_kelvin>(4.0),
            side: ThermalConductance::default(),
            top: ThermalConductance::new::<watt_per_kelvin>(6.0),
        };
        let adj_t = Adjacent {
            bottom: t(278.0),
            side: t(275.0),
            top: t(273.0),
        };

        let cond_deriv = layer.derivative_from_conduction(ua, adj_t);
        assert_relative_eq!(cond_deriv.value, 0.0);
    }

    #[test]
    fn conduction_superposition_all_faces() {
        // UA_b=5, UA_s=7, UA_t=9; ΔT_b=+1, ΔT_s=-2, ΔT_t=+3:
        // Q_dot= 5*1 + 7*(-2) + 9*3 = 5 - 14 + 27 = 18 W;  C = 6 J/K => dT/dt = 3 K/s
        let layer = Layer::new(
            t(400.0),
            Mass::new::<kilogram>(1.0),
            SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(6.0),
        );
        let ua = Adjacent {
            bottom: ThermalConductance::new::<watt_per_kelvin>(5.0),
            side: ThermalConductance::new::<watt_per_kelvin>(7.0),
            top: ThermalConductance::new::<watt_per_kelvin>(9.0),
        };
        let adj_t = Adjacent {
            bottom: t(401.0),
            side: t(398.0),
            top: t(403.0),
        };

        let cond_deriv = layer.derivative_from_conduction(ua, adj_t);
        assert_relative_eq!(cond_deriv.value, 3.0);
    }
}
