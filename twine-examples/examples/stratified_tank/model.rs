use std::convert::Infallible;

use jiff::civil::{DateTime, Time};
use twine_components::{
    controller::{
        SwitchState,
        thermostat::{SetpointThermostat, SetpointThermostatInput},
    },
    schedule::step_schedule::StepSchedule,
    thermal::stratified_tank_new::{
        Environment, PortFlow, StratifiedTank, StratifiedTankInput, StratifiedTankOutput,
    },
};
use twine_core::{Model, constraint::Constrained};
use twine_thermo::HeatFlow;
use uom::si::{
    f64::{Power, TemperatureInterval, ThermodynamicTemperature, VolumeRate},
    power::kilowatt,
    temperature_interval::degree_celsius as delta_celsius,
    thermodynamic_temperature::degree_celsius,
};

use super::{DEADBAND_C, ELEMENT_KW, ELEMENT_LOCATION, SETPOINT_C};

#[derive(Debug)]
pub(super) struct TankModel<const N: usize> {
    pub(super) tank: StratifiedTank<N, 1, 1>,
    pub(super) daily_draw_schedule: StepSchedule<Time, VolumeRate>,
}

/// Model input at each simulation step.
#[derive(Debug, Clone)]
pub(super) struct ModelInput<const N: usize> {
    pub(super) datetime: DateTime,
    pub(super) element_state: SwitchState,
    pub(super) t_ground: ThermodynamicTemperature,
    pub(super) t_room: ThermodynamicTemperature,
    pub(super) t_tank: [ThermodynamicTemperature; N],
}

/// Model output at each simulation step.
#[derive(Debug, Clone)]
pub(super) struct ModelOutput<const N: usize> {
    pub(super) draw: VolumeRate,
    pub(super) element_state: SwitchState,
    pub(super) tank: StratifiedTankOutput<N>,
}

impl<const N: usize> Model for TankModel<N> {
    type Input = ModelInput<N>;
    type Output = ModelOutput<N>;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Determine the current draw.
        let draw = Constrained::new(
            self.daily_draw_schedule
                .value_at(&input.datetime.time())
                .copied()
                .unwrap_or_default(),
        )
        .unwrap();

        // Call the thermostat component.
        let element_state = SetpointThermostat::heating(SetpointThermostatInput {
            state: input.element_state,
            // TODO: This should really check against the stable temperatures (tank output),
            //       not the temperature guesses (tank input).
            //       That will require adding a method to the tank model that returns just the
            //       stable temperatures, and another method that just computes the derivatives.
            //       Which I think we should do, but not right now.
            temperature: input.t_tank[ELEMENT_LOCATION],
            setpoint: ThermodynamicTemperature::new::<degree_celsius>(SETPOINT_C),
            deadband: TemperatureInterval::new::<delta_celsius>(DEADBAND_C),
        });

        // Call the tank component.
        let tank_output = self.tank.call(&StratifiedTankInput {
            temperatures: input.t_tank,
            port_flows: [PortFlow {
                rate: draw,
                inlet_temperature: input.t_ground,
            }],
            aux_heat_flows: [match input.element_state {
                SwitchState::Off => HeatFlow::None,
                SwitchState::On => {
                    HeatFlow::from_signed(Power::new::<kilowatt>(ELEMENT_KW)).unwrap()
                }
            }],
            environment: Environment {
                bottom: input.t_room,
                side: input.t_room,
                top: input.t_room,
            },
        });

        Ok(ModelOutput {
            draw: draw.into_inner(),
            element_state,
            tank: tank_output,
        })
    }
}
