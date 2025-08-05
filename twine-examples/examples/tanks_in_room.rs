//! # Tanks in Room Simulation
//!
//! This example simulates two thermal energy storage tanks in an environment
//! using first-order energy balance models and a forward Euler integrator.
//!
//! - The **first tank** receives fluid drawn from a cold source (e.g., ground).
//! - The **second tank** is connected in series, receiving fluid from the first.
//! - A time-varying draw schedule controls the flow rate through the tanks,
//!   allowing simulation of realistic hot water usage patterns.
//! - Heat loss to the environment is modeled using surface area and U-value.
//! - The first tank has an electric heating element with a thermostat.
//!
//! The simulation tracks the evolution of fluid temperatures in each tank and
//! plots them over time alongside ground and ambient temperatures, as well as
//! the draw flow rate.
//!
//! ## Running the Example
//!
//! To run this example with Cargo:
//!
//! ```sh
//! cargo run --example tanks_in_room
//! ```

use std::{convert::Infallible, time::Duration};

use jiff::civil::{DateTime, Time};
use twine_components::{
    controller::{
        SwitchState,
        thermostat::{SetpointThermostat, SetpointThermostatInput},
    },
    schedule::step_schedule::StepSchedule,
    thermal::tank::{Tank, TankConfig, TankInput, TankOutput},
};
use twine_core::{
    DurationExt, Model, Simulation, State, TimeIntegrable,
    constraint::{Constrained, StrictlyPositive},
};
use twine_plot::PlotApp;
use twine_thermo::{
    HeatFlow, Stream,
    fluid::Water,
    model::{
        StateFrom,
        incompressible::{Incompressible, IncompressibleFluid},
    },
};
use uom::{
    ConstZero,
    si::{
        area::square_foot,
        f64::{
            Area, HeatTransfer, MassRate, Power, TemperatureInterval, ThermodynamicTemperature,
            Volume, VolumeRate,
        },
        heat_transfer::watt_per_square_meter_kelvin,
        mass_rate::kilogram_per_minute,
        power::kilowatt,
        temperature_interval::degree_celsius as delta_celsius,
        thermodynamic_temperature::degree_celsius,
        volume::gallon,
        volume_rate::gallon_per_minute,
    },
};

/// Thermostat temperature setpoint for the first tank, in °C.
const SETPOINT_C: f64 = 50.0;

/// Thermostat deadband width, in °C.
const DEADBAND_C: f64 = 8.0;

/// Rated power of the first tank's electric heating element, in kW.
const ELEMENT_KW: f64 = 4.5;

/// A pair of tanks connected in series.
///
/// The first tank receives fluid from a cold source (e.g., ground water),
/// optional external heating from an element controlled by a thermostat,
/// and loses heat to the surrounding room.
///
/// The second tank draws from the first tank and also loses heat to the environment,
/// but does not receive external heating.
///
/// Fluid is drawn through both tanks according to a time-varying schedule,
/// defined using step functions of volumetric flow rates.
/// The schedule determines how much fluid is drawn at different times of day,
/// enabling simulation of usage patterns such as residential hot water demand.
#[derive(Debug)]
struct TanksInRoom<'a> {
    first_tank: Tank<'a, Water, Incompressible>,
    second_tank: Tank<'a, Water, Incompressible>,
    daily_draw_schedule: StepSchedule<Time, VolumeRate>,
}

/// Model input at each simulation step.
#[derive(Debug, Clone)]
struct Input {
    datetime: DateTime,
    element_state: SwitchState,
    t_ground: ThermodynamicTemperature,
    t_room: ThermodynamicTemperature,
    t_first_tank: ThermodynamicTemperature,
    t_second_tank: ThermodynamicTemperature,
}

/// Model output at each simulation step.
#[derive(Debug, Clone)]
struct Output {
    draw: Option<Constrained<MassRate, StrictlyPositive>>,
    element_state: SwitchState,
    first_tank: TankOutput<Water>,
    second_tank: TankOutput<Water>,
}

impl TanksInRoom<'_> {
    /// Creates a new two-tank system.
    fn new(
        first_tank_config: TankConfig,
        second_tank_config: TankConfig,
        daily_draw_schedule: StepSchedule<Time, VolumeRate>,
    ) -> Self {
        Self {
            first_tank: Tank::new(first_tank_config, &Incompressible).unwrap(),
            second_tank: Tank::new(second_tank_config, &Incompressible).unwrap(),
            daily_draw_schedule,
        }
    }
}

impl Model for TanksInRoom<'_> {
    type Input = Input;
    type Output = Output;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Call the schedule component and convert volumetric draw to a mass flow rate.
        let draw = self
            .daily_draw_schedule
            .value_at(&input.datetime.time())
            .map(|&draw| {
                let m_dot = draw * Water.reference_density();
                Constrained::new(m_dot).unwrap()
            });

        // Call the thermostat component.
        let element_state = SetpointThermostat::heating(SetpointThermostatInput {
            state: input.element_state,
            temperature: input.t_first_tank,
            setpoint: ThermodynamicTemperature::new::<degree_celsius>(SETPOINT_C),
            deadband: TemperatureInterval::new::<delta_celsius>(DEADBAND_C),
        });

        // Call the first tank component.
        let first_tank = self
            .first_tank
            .call(TankInput {
                ambient_temperature: input.t_room,
                aux_heat_flow: match element_state {
                    SwitchState::Off => HeatFlow::None,
                    SwitchState::On => {
                        HeatFlow::from_signed(Power::new::<kilowatt>(ELEMENT_KW)).unwrap()
                    }
                },
                inflow: draw.map(|m_dot| {
                    let state = Incompressible.state_from(input.t_ground).unwrap();
                    Stream::from_constrained(m_dot, state)
                }),
                state: Incompressible.state_from(input.t_first_tank).unwrap(),
            })
            .unwrap();

        // Call the second tank component.
        let second_tank = self
            .second_tank
            .call(TankInput {
                ambient_temperature: input.t_room,
                aux_heat_flow: HeatFlow::None,
                inflow: draw.map(|m_dot| {
                    let state = Incompressible.state_from(input.t_first_tank).unwrap();
                    Stream::from_constrained(m_dot, state)
                }),
                state: Incompressible.state_from(input.t_second_tank).unwrap(),
            })
            .unwrap();

        Ok(Output {
            draw,
            element_state,
            first_tank,
            second_tank,
        })
    }
}

/// A simulation of the `TanksInRoom` model using forward Euler integration.
#[derive(Debug)]
struct TanksInRoomSim<'a> {
    model: TanksInRoom<'a>,
}

impl<'a> Simulation<TanksInRoom<'a>> for TanksInRoomSim<'a> {
    type StepError = Infallible;

    fn model(&self) -> &TanksInRoom<'a> {
        &self.model
    }

    fn advance_time(
        &mut self,
        state: &State<TanksInRoom<'a>>,
        dt: Duration,
    ) -> Result<Input, Self::StepError> {
        let State { input, output } = state;

        Ok(Input {
            datetime: input.datetime + dt,
            element_state: output.element_state,
            t_first_tank: input
                .t_first_tank
                .step(output.first_tank.state_derivative.temperature, dt.as_time()),
            t_second_tank: input.t_second_tank.step(
                output.second_tank.state_derivative.temperature,
                dt.as_time(),
            ),
            ..input.clone()
        })
    }
}

/// A convenience struct for collecting time series data.
///
/// Stores temperature traces for ground water, room air, and both tanks,
/// as well as the draw rate, formatted for plotting.
/// Each series contains `(time, value)` pairs where:
///
/// - Time is in hours
/// - Temperature is in °C
/// - Draw is in kg/min
#[derive(Debug, Default)]
struct PlotSeries {
    ground: Vec<[f64; 2]>,
    room: Vec<[f64; 2]>,
    first_tank: Vec<[f64; 2]>,
    second_tank: Vec<[f64; 2]>,
    draw: Vec<[f64; 2]>,
}

impl PlotSeries {
    /// Appends the current values to each plot series.
    fn push(&mut self, State { input, output }: &State<TanksInRoom>) {
        let elapsed_hr = input
            .datetime
            .duration_since(DateTime::default())
            .as_secs_f64()
            / 3600.0;

        self.ground
            .push([elapsed_hr, input.t_ground.get::<degree_celsius>()]);
        self.room
            .push([elapsed_hr, input.t_room.get::<degree_celsius>()]);
        self.first_tank
            .push([elapsed_hr, input.t_first_tank.get::<degree_celsius>()]);
        self.second_tank
            .push([elapsed_hr, input.t_second_tank.get::<degree_celsius>()]);

        self.draw.push([
            elapsed_hr,
            output
                .draw
                .map_or(0.0, |draw| draw.into_inner().get::<kilogram_per_minute>()),
        ]);
    }
}

/// Runs the two-tank simulation and displays a temperature plot over time.
fn main() {
    // Set the model parameters.
    let model = TanksInRoom::new(
        TankConfig {
            volume: Volume::new::<gallon>(80.0),
            area: Area::new::<square_foot>(15.0),
            u_value: HeatTransfer::ZERO,
        },
        TankConfig {
            volume: Volume::new::<gallon>(120.0),
            area: Area::new::<square_foot>(25.0),
            u_value: HeatTransfer::new::<watt_per_square_meter_kelvin>(0.1),
        },
        StepSchedule::new([
            // From 9 to 10:30 am there is a steady 0.2 GPM draw.
            (
                Time::constant(9, 0, 0, 0)..Time::constant(10, 30, 0, 0),
                VolumeRate::new::<gallon_per_minute>(0.2),
            )
                .try_into()
                .unwrap(),
            // At 6 pm there is a 15 minute draw at 2.5 GPM.
            (
                Time::constant(18, 0, 0, 0)..Time::constant(18, 15, 0, 0),
                VolumeRate::new::<gallon_per_minute>(2.5),
            )
                .try_into()
                .unwrap(),
        ])
        .unwrap(),
    );

    // Create the simulation.
    let sim = TanksInRoomSim { model };

    // Specify the initial conditions.
    let initial_conditions = Input {
        datetime: DateTime::default(),
        t_ground: ThermodynamicTemperature::new::<degree_celsius>(10.0),
        t_room: ThermodynamicTemperature::new::<degree_celsius>(20.0),
        t_first_tank: ThermodynamicTemperature::new::<degree_celsius>(60.0),
        t_second_tank: ThermodynamicTemperature::new::<degree_celsius>(50.0),
        element_state: SwitchState::Off,
    };

    // Run the simulation for five days with a one minute time step.
    let mut series = PlotSeries::default();
    for step_result in sim
        .into_step_iter(initial_conditions, Duration::from_secs(60))
        .take(7500)
    {
        let state = step_result.expect("Step should succeed");
        series.push(&state);
    }

    // Plot the temperatures.
    let app = PlotApp::new()
        .add_series("Ground Water Temp [°C]", &series.ground)
        .add_series("Room Temp [°C]", &series.room)
        .add_series("First Tank Temp [°C]", &series.first_tank)
        .add_series("Second Tank Temp [°C]", &series.second_tank)
        .add_series("Draw [kg/min]", &series.draw);

    app.run("Tanks in Room Example").unwrap();
}
