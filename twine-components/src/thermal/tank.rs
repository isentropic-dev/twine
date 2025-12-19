use std::marker::PhantomData;

use twine_core::{
    TimeIntegrable,
    constraint::{Constrained, ConstraintError, NonNegative, StrictlyPositive},
};
use twine_thermo::{
    BoundaryFlow, ControlVolume, HeatFlow, MassFlow, PropertyError, State, StateDerivative, Stream,
    capability::{HasCv, HasEnthalpy, HasInternalEnergy, ThermoModel},
    units::TemperatureDifference,
};
use uom::si::f64::{Area, HeatTransfer, ThermodynamicTemperature, Volume};

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
pub struct Tank<'a, Fluid, Model> {
    pub area: Constrained<Area, StrictlyPositive>,
    pub u_value: Constrained<HeatTransfer, NonNegative>,
    pub volume: Constrained<Volume, StrictlyPositive>,
    pub model: &'a Model,
    _marker: PhantomData<Fluid>,
}

impl<'a, Fluid, Model> Tank<'a, Fluid, Model> {
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

    /// Optional fluid inflow to the tank.
    ///
    /// If present, a matching outflow is assumed to maintain constant mass.
    pub inflow: Option<Stream<Fluid>>,

    /// Thermodynamic state of the fluid in the tank.
    pub state: State<Fluid>,
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

impl<Model> Tank<'_, Model::Fluid, Model>
where
    Model: ThermoModel + HasCv + HasEnthalpy + HasInternalEnergy,
    Model::Fluid: TimeIntegrable<Derivative = ()>,
{
    /// Call the tank component.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if a required thermodynamic property cannot be computed.
    ///
    /// # Panics
    ///
    /// Panics if `U·A·ΔT` is not finite.
    pub fn call(
        &self,
        input: TankInput<Model::Fluid>,
    ) -> Result<TankOutput<Model::Fluid>, PropertyError> {
        let TankInput {
            ambient_temperature,
            aux_heat_flow,
            inflow,
            state,
        } = input;

        // Heat loss to (or gain from) the ambient environment.
        let ambient_heat_flow = HeatFlow::from_signed(
            self.u_value.into_inner()
                * self.area.into_inner()
                * ambient_temperature.minus(state.temperature),
        )
        .expect("U·A·ΔT is always finite");

        let (mass_flow_in, mass_flow_out) = match inflow {
            Some(stream) => MassFlow::balanced_pair(stream),
            None => (MassFlow::None, MassFlow::None),
        };

        let state_derivative = ControlVolume::from_constrained(self.volume, state)
            .state_derivative(
                &[
                    BoundaryFlow::Mass(mass_flow_in),
                    BoundaryFlow::Mass(mass_flow_out),
                    BoundaryFlow::Heat(aux_heat_flow),
                    BoundaryFlow::Heat(ambient_heat_flow),
                ],
                self.model,
            )?;

        Ok(TankOutput {
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
        capability::StateFrom, fluid::Water, model::incompressible::Incompressible,
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
    fn water_tank(model: &Incompressible<Water>) -> Tank<'_, Water, Incompressible<Water>> {
        Tank::new(
            TankConfig {
                volume: Volume::new::<gallon>(80.0),
                area: Area::new::<square_foot>(9.0),
                u_value: HeatTransfer::new::<watt_per_square_meter_kelvin>(0.1),
            },
            model,
        )
        .unwrap()
    }

    /// Returns a baseline [`TankInput`] at equilibrium conditions.
    ///
    /// Equilibrium conditions are:
    ///
    /// - Tank and ambient temperatures are 20°C
    /// - No auxiliary heat input
    /// - No inflow
    fn equilibrium_input(model: &Incompressible<Water>) -> TankInput<Water> {
        let t_ambient = ThermodynamicTemperature::new::<degree_celsius>(20.0);
        let state = model.state_from(t_ambient).unwrap();

        TankInput {
            ambient_temperature: t_ambient,
            aux_heat_flow: HeatFlow::None,
            inflow: None,
            state,
        }
    }

    #[test]
    fn nothing_happens_at_equilibrium() {
        let thermo = Incompressible::<Water>::new().unwrap();
        let tank = water_tank(&thermo);
        let input = equilibrium_input(&thermo);
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
        let thermo = Incompressible::<Water>::new().unwrap();
        let tank = water_tank(&thermo);
        let input = TankInput {
            state: thermo
                .state_from(ThermodynamicTemperature::new::<degree_celsius>(50.0))
                .unwrap(),
            ..equilibrium_input(&thermo)
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
        let thermo = Incompressible::<Water>::new().unwrap();
        let tank = water_tank(&thermo);

        let input_without_draw = TankInput {
            aux_heat_flow: HeatFlow::from_signed(Power::new::<kilowatt>(4.5)).unwrap(),
            ..equilibrium_input(&thermo)
        };

        let output = tank.call(input_without_draw).unwrap();
        assert!(
            output.state_derivative.temperature.value > 0.0,
            "Expected tank to be heating"
        );

        let input_with_draw = TankInput {
            inflow: Some(
                Stream::new(
                    MassRate::new::<kilogram_per_second>(0.4),
                    thermo
                        .state_from(ThermodynamicTemperature::new::<degree_celsius>(10.0))
                        .unwrap(),
                )
                .unwrap(),
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
