use std::marker::PhantomData;

use twine_core::{
    Component, TimeIntegrable,
    constraint::{Constrained, ConstraintError, NonNegative, StrictlyPositive},
};
use twine_thermo::{
    BoundaryFlow, ControlVolume, HeatFlow, MassFlow, PropertyError, State, StateDerivative,
    model::ThermodynamicProperties, units::TemperatureDifference,
};
use uom::si::f64::{Area, HeatTransfer, MassRate, ThermodynamicTemperature, Volume};

/// A fully mixed thermal energy storage tank.
///
/// Represents a spatially uniform control volume of fluid with thermal capacity,
/// which exchanges heat with its environment and supports mass flow.
/// Inlet and outlet mass flow rates are equal, keeping the fluid mass constant.
/// The tank's volume and heat transfer area are fixed.
///
/// Energy is gained or lost through:
///
/// - Fluid flow
/// - Auxiliary heat addition or extraction (e.g., via a heater or chiller)
/// - Heat loss (or gain) with ambient via a U·A·ΔT model
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tank<'a, Fluid, Model: ThermodynamicProperties<Fluid>> {
    pub area: Constrained<Area, StrictlyPositive>,
    pub u_value: Constrained<HeatTransfer, NonNegative>,
    pub volume: Constrained<Volume, StrictlyPositive>,
    pub model: &'a Model,
    _marker: PhantomData<Fluid>,
}

impl<'a, Fluid, Model: ThermodynamicProperties<Fluid>> Tank<'a, Fluid, Model> {
    /// Creates a new `Tank` from configuration and a model.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if any configuration parameter is invalid.
    pub fn new(config: TankConfig, model: &'a Model) -> Result<Self, ConstraintError> {
        let TankConfig {
            area,
            u_value,
            volume,
        } = config;

        Ok(Self {
            area: Constrained::new(area)?,
            u_value: Constrained::new(u_value)?,
            volume: Constrained::new(volume)?,
            model,
            _marker: PhantomData,
        })
    }
}

/// Geometry and heat transfer characteristics of the tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankConfig {
    pub area: Area,
    pub u_value: HeatTransfer,
    pub volume: Volume,
}

/// Inputs required to evaluate the tank's thermal response.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TankInput<Fluid> {
    /// Ambient temperature surrounding the tank.
    pub ambient_temperature: ThermodynamicTemperature,

    /// Auxiliary heat addition or extraction (e.g., from a heater or chiller).
    pub aux_heat_flow: HeatFlow,

    /// Thermodynamic state of the incoming fluid.
    pub inlet_state: State<Fluid>,

    /// Optional mass flow through the tank.
    ///
    /// If present, flow enters at `inlet_state` and exits at `tank_state`.
    /// Inlet and outlet flow rates are equal, maintaining constant fluid mass.
    pub mass_flow_rate: Option<Constrained<MassRate, StrictlyPositive>>,

    /// Thermodynamic state of the fluid in the tank.
    pub tank_state: State<Fluid>,
}

/// Outputs describing the tank's instantaneous thermal response.
#[derive(Debug, Clone, PartialEq)]
pub struct TankOutput<Fluid: TimeIntegrable> {
    /// Heat exchange with the ambient environment.
    ///
    /// - [`HeatFlow::In`] if heat flows into the tank (ambient is warmer)
    /// - [`HeatFlow::Out`] if heat flows out of the tank (tank is warmer)
    /// - [`HeatFlow::None`] if no heat transfer occurs
    pub ambient_heat_flow: HeatFlow,

    /// Time derivative of the tank's internal fluid state.
    pub state_derivative: StateDerivative<Fluid>,
}

impl<Fluid, Model> Component for Tank<'_, Fluid, Model>
where
    Model: ThermodynamicProperties<Fluid>,
    Fluid: TimeIntegrable<Derivative = ()>,
{
    type Input = TankInput<Fluid>;
    type Output = TankOutput<Fluid>;
    type Error = PropertyError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let TankInput {
            ambient_temperature,
            aux_heat_flow,
            inlet_state,
            mass_flow_rate,
            tank_state,
        } = input;

        // Heat loss to (or gain from) the ambient environment.
        let ambient_heat_flow = HeatFlow::from_signed(
            self.u_value.into_inner()
                * self.area.into_inner()
                * ambient_temperature.minus(tank_state.temperature),
        )
        .expect("U·A·ΔT is always finite");

        // Create paired mass flows (if present) to maintain constant mass.
        let (mass_flow_in, mass_flow_out) = match mass_flow_rate {
            Some(rate) => (MassFlow::In(rate, inlet_state), MassFlow::Out(rate)),
            None => (MassFlow::None, MassFlow::None),
        };

        let state_derivative = ControlVolume::from_constrained(self.volume, tank_state)
            .state_derivative(
                &[
                    BoundaryFlow::Mass(mass_flow_in),
                    BoundaryFlow::Mass(mass_flow_out),
                    BoundaryFlow::Heat(aux_heat_flow),
                    BoundaryFlow::Heat(ambient_heat_flow),
                ],
                self.model,
            )?;

        Ok(Self::Output {
            ambient_heat_flow,
            state_derivative,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use twine_core::TimeDerivative;
    use twine_thermo::{
        fluid::Water,
        model::{StateFrom, incompressible::Incompressible},
    };
    use uom::{
        ConstZero,
        si::{
            area::square_foot,
            f64::{MassDensity, MassRate, Power},
            heat_transfer::watt_per_square_meter_kelvin,
            mass_rate::kilogram_per_second,
            power::{kilowatt, watt},
            thermodynamic_temperature::degree_celsius,
            volume::gallon,
        },
    };

    /// Returns a default tank filled with water.
    ///
    /// Uses volume, surface area, and U-value typical of a residential water heater.
    fn water_tank() -> Tank<'static, Water, Incompressible> {
        Tank::new(
            TankConfig {
                volume: Volume::new::<gallon>(80.0),
                area: Area::new::<square_foot>(9.0),
                u_value: HeatTransfer::new::<watt_per_square_meter_kelvin>(0.1),
            },
            &Incompressible,
        )
        .unwrap()
    }

    /// Returns a baseline [`TankInput`] at equilibrium conditions.
    ///
    /// Equilibrium conditions are:
    ///
    /// - Tank, inlet, and ambient temperatures are 20°C
    /// - No auxilliary heat input
    /// - No mass flow
    fn equilibrium_input() -> TankInput<Water> {
        let t_ambient = ThermodynamicTemperature::new::<degree_celsius>(20.0);
        let state = Incompressible.state_from(t_ambient).unwrap();

        TankInput {
            ambient_temperature: t_ambient,
            aux_heat_flow: HeatFlow::None,
            inlet_state: state,
            mass_flow_rate: None,
            tank_state: state,
        }
    }

    #[test]
    fn nothing_happens_at_equilibrium() {
        let tank = water_tank();
        let input = equilibrium_input();
        let output = tank.call(input).unwrap();

        assert_eq!(output.ambient_heat_flow.signed(), Power::ZERO);
        assert_eq!(
            output.state_derivative,
            StateDerivative {
                temperature: TimeDerivative::<ThermodynamicTemperature>::ZERO,
                density: TimeDerivative::<MassDensity>::ZERO,
                fluid: (),
            }
        );
    }

    #[test]
    fn tank_loses_heat_when_hotter_than_ambient() {
        let tank = water_tank();
        let input = TankInput {
            tank_state: Incompressible
                .state_from(ThermodynamicTemperature::new::<degree_celsius>(50.0))
                .unwrap(),
            ..equilibrium_input()
        };
        let output = tank.call(input).unwrap();

        assert_relative_eq!(
            output.ambient_heat_flow.signed().get::<watt>(),
            -2.5084,
            epsilon = 1e-4
        );
        assert!(
            output.state_derivative.temperature.value < 0.0,
            "Expected tank to be cooling"
        );
    }

    #[test]
    fn tank_with_aux_heating_heats_without_flow_and_cools_with_flow() {
        let tank = water_tank();

        let input_without_draw = TankInput {
            tank_state: Incompressible
                .state_from(ThermodynamicTemperature::new::<degree_celsius>(50.0))
                .unwrap(),
            aux_heat_flow: HeatFlow::from_signed(Power::new::<kilowatt>(4.5)).unwrap(),
            ..equilibrium_input()
        };

        let output = tank.call(input_without_draw).unwrap();
        assert!(
            output.state_derivative.temperature.value > 0.0,
            "Expected tank to be heating"
        );

        let input_with_draw = TankInput {
            mass_flow_rate: Some(
                Constrained::new(MassRate::new::<kilogram_per_second>(0.4)).unwrap(),
            ),
            ..input_without_draw
        };

        let output = tank.call(input_with_draw).unwrap();
        assert!(
            output.state_derivative.temperature.value < 0.0,
            "Expected tank to be cooling"
        );
    }
}
