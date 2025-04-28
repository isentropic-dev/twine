use thiserror::Error;
use twine_core::{
    thermo::{
        fluid::{
            CvProvider, DensityProvider, EnthalpyProvider, FluidPropertyError, FluidPropertyModel,
            TemperatureProvider,
        },
        units::{temperature_difference, TemperatureRate, UValue},
    },
    Component,
};
use uom::si::f64::{Area, MassRate, Power, ThermodynamicTemperature, Volume};

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
/// dT/dt = (m_dot · (h_in - h_out) + Q_dot_in - Q_dot_loss) / (m · c_v)
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
/// - `c_v`        = specific heat at constant volume of the fluid (J/kg·K)
///
/// The energy balance accounts for three contributions:
/// - Enthalpy change due to mass flow through the tank
/// - External heating applied directly to the fluid
/// - Heat losses to the ambient environment through the tank walls
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
    /// Surface area available for heat exchange with the environment.
    pub area: Area,

    /// Fluid property model used to evaluate thermodynamic properties.
    pub fluid: F,

    /// Overall heat transfer coefficient (U-value) of the tank.
    ///
    /// Used to model heat loss to the environment through the tank's surface
    /// based on the temperature difference between the fluid and ambient conditions.
    pub u_value: UValue,

    /// Total internal volume of the tank.
    pub volume: Volume,
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
    pub mass_flow_rate: MassRate,

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
    pub tank_temperature_derivative: TemperatureRate,
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
    F: FluidPropertyModel + TemperatureProvider + DensityProvider + EnthalpyProvider + CvProvider,
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
        // Rate of heat transfer from the tank fluid to the environment.
        let heat_loss = self.u_value
            * self.area
            * temperature_difference(
                self.fluid.temperature(&input.tank_state),
                input.ambient_temperature,
            );

        // Net rate of heat transfer into the tank fluid.
        let h_in = self.fluid.enthalpy(&input.inlet_state);
        let h_out = self.fluid.enthalpy(&input.tank_state);
        let q_dot_net = input.mass_flow_rate * (h_in - h_out) + input.heat_input - heat_loss;

        // Fluid temperature derivative based on the energy balance.
        let m = self.volume * self.fluid.density(&input.tank_state);
        let c_v = self.fluid.cv(&input.tank_state)?;
        let tank_temperature_derivative = q_dot_net / (m * c_v);

        Ok(TankOutput {
            heat_loss,
            tank_temperature_derivative,
        })
    }
}
