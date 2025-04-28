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
/// Models a single, spatially uniform control volume of fluid that exchanges
/// heat with its environment and allows mass flow into and out of the tank.
/// The model assumes equal mass flow rates at the inlet and outlet. The tank
/// volume is fixed. The mass of fluid inside the tank is determined from the
/// fluid density at the current state and is treated as constant during the
/// evaluation of the temperature derivative.
///
/// The energy balance tracks:
/// - Incoming and outgoing fluid enthalpy
/// - External heat input
/// - Heat loss to the surroundings (via `U · A · ΔT`)
///
/// # Type Parameters
///
/// - `F`: A fluid property model implementing [`FluidPropertyModel`] and related traits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tank<F: FluidPropertyModel> {
    pub volume: Volume,
    pub area: Area,
    pub u_value: UValue,
    pub fluid: F,
}

/// Input variables required to compute the tank's dynamic behavior.
///
/// Provides the current environmental conditions, inlet fluid conditions, and the current
/// thermodynamic state of the tank fluid.
///
/// # Type Parameters
///
/// - `F`: A fluid property model implementing [`FluidPropertyModel`].
///
/// # Fields
///
/// - `ambient_temp`: Ambient temperature surrounding the tank (for heat loss calculation).
/// - `heat_input`: External heat input applied to the tank (positive adds energy).
/// - `inlet_state`: Thermodynamic state of the incoming fluid stream.
/// - `mass_flow_rate`: Mass flow rate of incoming fluid (kg/s).
/// - `tank_state`: Current thermodynamic state of the fluid within the tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankInput<F: FluidPropertyModel> {
    pub ambient_temp: ThermodynamicTemperature,
    pub heat_input: Power,
    pub inlet_state: F::State,
    pub mass_flow_rate: MassRate,
    pub tank_state: F::State,
}

/// Computed output variables describing the tank's thermal response.
///
/// Provides key dynamic properties derived from the energy balance at the current timestep.
///
/// # Fields
///
/// - `heat_loss`: Rate of heat loss from the tank to the ambient environment (W).
/// - `tank_temp_derivative`: Rate of change of the tank fluid temperature (K/s).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankOutput {
    pub heat_loss: Power,
    pub tank_temp_derivative: TemperatureRate,
}

/// Errors that can occur when evaluating the tank's energy balance.
///
/// Represents failures during fluid property evaluation needed to compute thermal behavior,
/// such as an undefined specific heat capacity.
///
/// # Variants
///
/// - `Fluid`: An error occurred when querying fluid properties (e.g., invalid state or
///   undefined specific heat).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TankError {
    /// Failed to evaluate fluid properties.
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

    /// Solves the transient energy balance for a fully mixed tank.
    ///
    /// The governing equation is:
    ///
    /// ```text
    /// dT/dt = (ṁ · (h_in - h_out) + Q_in - Q_loss) / (m · c_v)
    /// ```
    ///
    /// where:
    /// - `h_in`   = specific enthalpy of incoming fluid
    /// - `h_out`  = specific enthalpy of fluid leaving the tank (equal to tank fluid enthalpy)
    /// - `Q_in`   = external heat input
    /// - `Q_loss` = heat loss to ambient via surface transfer (`U·A·ΔT`)
    /// - `m`      = total fluid mass inside the tank
    /// - `c_v`    = specific heat at constant volume
    ///
    /// # Errors
    ///
    /// Returns a [`TankError`] if fluid property evaluation (such as specific
    /// heat capacity) fails at the current tank state.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Heat lost to surroundings.
        let heat_loss = self.u_value
            * self.area
            * temperature_difference(
                self.fluid.temperature(&input.tank_state),
                input.ambient_temp,
            );

        // Energy balance on fluid flow.
        let h_in = self.fluid.enthalpy(&input.inlet_state);
        let h_tank = self.fluid.enthalpy(&input.tank_state);
        let inlet_heat_flow = input.mass_flow_rate * (h_in - h_tank);

        // Total mass and specific heat in the tank: used as thermal capacitance
        let mass = self.volume * self.fluid.density(&input.tank_state);
        let cv = self.fluid.cv(&input.tank_state)?;

        // Energy balance: total net heat flow over thermal capacitance
        let tank_temp_derivative = (input.heat_input + inlet_heat_flow - heat_loss) / (mass * cv);

        Ok(TankOutput {
            heat_loss,
            tank_temp_derivative,
        })
    }
}
