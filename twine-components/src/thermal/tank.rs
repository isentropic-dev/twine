use thiserror::Error;
use twine_core::{
    Component, TimeDerivative,
    thermo::{
        fluid::{
            FluidPropertyError, FluidPropertyModel, ProvidesCv, ProvidesDensity, ProvidesEnthalpy,
            ProvidesTemperature,
        },
        units::{PositiveMassRate, TemperatureDifference},
    },
};
use uom::si::f64::{Area, HeatTransfer, Power, ThermodynamicTemperature, Volume};

/// A fully mixed thermal energy storage tank.
///
/// Represents a single, spatially uniform control volume of fluid that
/// exchanges heat with its environment and allows mass flow through it.
/// The mass flow rates at the inlet and outlet are assumed to be equal,
/// ensuring constant fluid mass during the energy balance. An external
/// heat input can also be specified, representing energy added directly
/// to the tank contents.
///
/// The model solves the transient energy balance:
///
/// ```text
/// dT/dt = (m_dot · (h_in - h_out) + Q_dot_in - Q_dot_loss) / (m · cv)
/// ```
///
/// where (shown in SI units for clarity):
/// - `dT/dt`      = rate of change of the tank fluid temperature (K/s)
/// - `m_dot`      = mass flow rate through the tank (kg/s)
/// - `h_in`       = specific enthalpy of the incoming fluid (J/kg)
/// - `h_out`      = specific enthalpy of the fluid leaving the tank (J/kg)
/// - `Q_dot_in`   = external heat input rate (W)
/// - `Q_dot_loss` = heat loss rate to the environment (W)
/// - `m`          = total mass of fluid inside the tank (kg)
/// - `cv`         = specific heat at constant volume of the fluid (J/kg·K)
///
/// The energy balance accounts for three contributions:
/// - Enthalpy change due to mass flow through the tank.
/// - External heating applied directly to the fluid.
/// - Heat losses to the ambient environment through the tank walls.
///
/// The tank volume and heat transfer area are treated as constant.
/// The fluid mass is determined from the fluid density at the current tank
/// state and is treated as constant during evaluation of the instantaneous
/// temperature derivative.
///
/// Note that the SI units shown above are for illustration purposes only.
/// Units for all input and output values are automatically enforced and
/// converted as necessary by the `uom` type system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tank<F: FluidPropertyModel> {
    /// Physical and thermal configuration of the tank.
    ///
    /// Includes the internal volume, external surface area,
    /// and heat transfer characteristics (e.g., insulation).
    pub config: TankConfig,

    /// Fluid property model used to evaluate thermodynamic properties.
    pub fluid: F,
}

/// Describes the tank's size and thermal loss characteristics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankConfig {
    pub area: Area,
    pub u_value: HeatTransfer,
    pub volume: Volume,
}

impl<F: FluidPropertyModel> Tank<F> {
    /// Creates a new tank from the given fluid and configuration.
    pub fn new(fluid: F, config: TankConfig) -> Self {
        Self { config, fluid }
    }
}

/// Inputs required to evaluate the tank's energy balance.
///
/// Specifies the ambient conditions, the inlet fluid state, and the current
/// tank fluid state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankInput<F: FluidPropertyModel> {
    /// Ambient temperature surrounding the tank, used to calculate heat loss.
    pub ambient_temperature: ThermodynamicTemperature,

    /// External heat input applied to the tank (positive values add energy).
    pub heat_input: Power,

    /// Thermodynamic state of the incoming fluid stream.
    pub inlet_state: F::State,

    /// Mass flow rate of fluid through the tank.
    ///
    /// Assumed equal at inlet and outlet to maintain constant tank mass.
    pub mass_flow_rate: PositiveMassRate,

    /// Current thermodynamic state of the fluid contained within the tank.
    pub tank_state: F::State,
}

/// Outputs representing the tank's thermal response at a given instant.
///
/// Provides the computed heat loss to the environment and the temperature
/// rate of change based on the current energy balance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankOutput {
    /// Rate of heat loss from the tank to the ambient environment.
    ///
    /// Positive if heat is leaving the tank.
    pub heat_loss: Power,

    /// Rate of change of the tank fluid temperature.
    ///
    /// Positive for heating, negative for cooling.
    pub tank_temperature_derivative: TimeDerivative<ThermodynamicTemperature>,
}

/// Errors that can occur while evaluating the tank's energy balance.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TankError {
    /// Failed to evaluate fluid properties at the current tank state.
    #[error("Fluid property evaluation failed: {0}")]
    Fluid(#[from] FluidPropertyError),
}

impl<F> Component for Tank<F>
where
    F: FluidPropertyModel + ProvidesTemperature + ProvidesDensity + ProvidesEnthalpy + ProvidesCv,
{
    type Input = TankInput<F>;
    type Output = TankOutput;
    type Error = TankError;

    /// Solves the transient energy balance for the tank based on the provided input.
    ///
    /// # Parameters
    ///
    /// - `input`: [`TankInput`] describing the current conditions and tank state.
    ///
    /// # Returns
    ///
    /// A [`TankOutput`] containing the heat loss and tank fluid temperature derivative.
    ///
    /// # Errors
    ///
    /// Returns a [`TankError`] if evaluation of the specific heat capacity fails at
    /// the current tank state.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let TankConfig {
            area,
            u_value,
            volume,
        } = self.config;

        // Rate of heat transfer from the tank fluid to the environment.
        let t_tank = self.fluid.temperature(&input.tank_state);
        let delta_t = t_tank.minus(input.ambient_temperature);
        let heat_loss = u_value * area * delta_t;

        // Net rate of heat transfer into the tank fluid.
        let m_dot = input.mass_flow_rate.into_inner();
        let h_in = self.fluid.enthalpy(&input.inlet_state);
        let h_out = self.fluid.enthalpy(&input.tank_state);
        let q_dot_net = m_dot * (h_in - h_out) + input.heat_input - heat_loss;

        // Rate of temperature change, derived from the net energy balance.
        let m = volume * self.fluid.density(&input.tank_state);
        let cv = self.fluid.cv(&input.tank_state)?;
        let tank_temperature_derivative = q_dot_net / (m * cv);

        Ok(TankOutput {
            heat_loss,
            tank_temperature_derivative,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        area::square_foot,
        heat_transfer::watt_per_square_meter_kelvin,
        mass_rate::kilogram_per_second,
        power::{kilowatt, watt},
        thermodynamic_temperature::degree_celsius,
        volume::gallon,
    };

    use crate::fluid::IncompressibleLiquid;

    /// Returns a default test tank filled with water.
    ///
    /// Configured with a fixed volume, surface area, and U-value typical of a
    /// residential water heater.
    fn water_tank() -> Tank<IncompressibleLiquid> {
        Tank::new(
            IncompressibleLiquid::water(),
            TankConfig {
                volume: Volume::new::<gallon>(80.0),
                area: Area::new::<square_foot>(9.0),
                u_value: HeatTransfer::new::<watt_per_square_meter_kelvin>(0.1),
            },
        )
    }

    /// Returns a baseline [`TankInput`] at equilibrium conditions.
    ///
    /// - Tank, inlet, and ambient temperatures all set to 20°C.
    /// - No external heat input.
    /// - Zero mass flow rate.
    ///
    /// Useful as a starting point for test cases.
    fn equilibrium_input() -> TankInput<IncompressibleLiquid> {
        let t_ambient = ThermodynamicTemperature::new::<degree_celsius>(20.0);
        TankInput {
            ambient_temperature: t_ambient,
            inlet_state: t_ambient,
            tank_state: t_ambient,
            heat_input: Power::default(),
            mass_flow_rate: PositiveMassRate::default(),
        }
    }

    #[test]
    fn nothing_happens_at_equilibrium() {
        let tank = water_tank();
        let input = equilibrium_input();
        let output = tank.call(input).unwrap();

        assert_relative_eq!(output.heat_loss.value, 0.0);
        assert_relative_eq!(output.tank_temperature_derivative.value, 0.0);
    }

    #[test]
    fn tank_cools_when_temp_above_ambient() {
        let tank = water_tank();
        let input = TankInput {
            tank_state: ThermodynamicTemperature::new::<degree_celsius>(50.0),
            ..equilibrium_input()
        };
        let output = tank.call(input).unwrap();

        assert_relative_eq!(output.heat_loss.get::<watt>(), 2.5084, epsilon = 1e-4);
        assert!(
            output.tank_temperature_derivative.value < 0.0,
            "Expected tank to be cooling"
        );
    }

    #[test]
    fn tank_heats_without_draw_and_cools_with_draw() {
        let tank = water_tank();

        let input_without_draw = TankInput {
            tank_state: ThermodynamicTemperature::new::<degree_celsius>(50.0),
            heat_input: Power::new::<kilowatt>(4.5),
            ..equilibrium_input()
        };
        let output = tank.call(input_without_draw).unwrap();
        assert!(
            output.tank_temperature_derivative.value > 0.0,
            "Expected tank to be heating"
        );

        let input_with_draw = TankInput {
            mass_flow_rate: PositiveMassRate::new::<kilogram_per_second>(0.4).unwrap(),
            ..input_without_draw
        };
        let output = tank.call(input_with_draw).unwrap();
        assert!(
            output.tank_temperature_derivative.value < 0.0,
            "Expected tank to be cooling"
        );
    }
}
