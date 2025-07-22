//! # Tanks in Room Simulation
//!
//! This example simulates two thermal energy storage tanks in an environment
//! using first-order energy balance models and a forward Euler integrator.
//!
//! - The **first tank** receives fluid drawn from a cold source (e.g., ground).
//! - The **second tank** is connected in series, receiving fluid from the first.
//! - A time-varying draw schedule determines the flow rate through both tanks.
//! - Heat loss to the environment is modeled using surface area and U-value.
//!
//! The simulation tracks the evolution of fluid temperatures in each tank and
//! plots them over time alongside ground and ambient temperatures.
//!
//! ## Running the Example
//!
//! To run this example with Cargo:
//!
//! ```sh
//! cargo run --example tanks_in_room
//! ```

use std::{convert::Infallible, time::Duration};

use twine_components::thermal::tank2::{Tank, TankConfig, TankInput, TankOutput};
use twine_core::{
    Component, DurationExt, Simulation, State, TimeIntegrable,
    constraint::{Constrained, StrictlyPositive},
};
use twine_plot::PlotApp;
use twine_thermo::{
    HeatFlow,
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
            Area, HeatTransfer, MassRate, Power, ThermodynamicTemperature, Time, Volume, VolumeRate,
        },
        heat_transfer::watt_per_square_meter_kelvin,
        power::watt,
        thermodynamic_temperature::degree_celsius,
        time::{hour, second},
        volume::gallon,
        volume_rate::gallon_per_minute,
    },
};

/// A pair of tanks connected in series, modeled as a single thermal component.
///
/// The first tank receives fluid from a cold source (e.g., ground water),
/// external heat input, and loses heat to the surrounding room.
/// The second tank draws from the first tank and is also subject to heat loss,
/// but does not receive external heating.
///
/// Fluid is drawn through both tanks according to a fixed hourly draw schedule,
/// defined as a 24-element array of volumetric flow rates.
/// The schedule determines how much fluid is drawn during each hour of the day,
/// enabling simulation of usage patterns such as residential hot water demand.
#[derive(Debug)]
struct TanksInRoom<'a> {
    first_tank: Tank<'a, Water, Incompressible>,
    second_tank: Tank<'a, Water, Incompressible>,
    draw_schedule: [VolumeRate; 24],
}

/// Model input at each simulation step.
#[derive(Debug, Clone)]
struct Input {
    time: Time,
    t_ground: ThermodynamicTemperature,
    t_room: ThermodynamicTemperature,
    t_first_tank: ThermodynamicTemperature,
    t_second_tank: ThermodynamicTemperature,
    q_dot_first_tank: Power,
}

/// Model output at each simulation step.
#[derive(Debug, Clone)]
struct Output {
    first_tank: TankOutput<Water>,
    second_tank: TankOutput<Water>,
}

impl TanksInRoom<'_> {
    /// Creates a new two-tank system using the provided fluid and tank configurations.
    ///
    /// The two tanks are connected in series and share the same fluid model.
    fn new(first_tank_config: TankConfig, second_tank_config: TankConfig) -> Self {
        Self {
            first_tank: Tank::new(first_tank_config, &Incompressible).unwrap(),
            second_tank: Tank::new(second_tank_config, &Incompressible).unwrap(),
            draw_schedule: [VolumeRate::ZERO; 24],
        }
    }

    /// Prepares the input for the first tank based on current simulation state.
    fn prepare_first_tank_input(&self, input: &Input) -> TankInput<Water> {
        TankInput {
            ambient_temperature: input.t_room,
            aux_heat_flow: HeatFlow::from_signed(input.q_dot_first_tank).unwrap(),
            inlet_state: Incompressible.state_from(input.t_ground).unwrap(),
            mass_flow_rate: self.draw_at_time(input.time),
            tank_state: Incompressible.state_from(input.t_first_tank).unwrap(),
        }
    }

    /// Prepares the input for the second tank based on current simulation state.
    fn prepare_second_tank_input(&self, input: &Input) -> TankInput<Water> {
        TankInput {
            ambient_temperature: input.t_room,
            aux_heat_flow: HeatFlow::None,
            inlet_state: Incompressible.state_from(input.t_first_tank).unwrap(),
            mass_flow_rate: self.draw_at_time(input.time),
            tank_state: Incompressible.state_from(input.t_second_tank).unwrap(),
        }
    }

    /// Returns the mass flow rate through the tanks at a given time.
    ///
    /// Flow is looked up from the draw schedule using the current hour of day,
    /// then converted from volumetric to mass flow using the water's default density.
    fn draw_at_time(&self, time: Time) -> Option<Constrained<MassRate, StrictlyPositive>> {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let hour_of_day = time.get::<hour>().floor() as usize % 24;

        let draw = self.draw_schedule[hour_of_day];
        if draw > VolumeRate::ZERO {
            let m_dot = draw * Water.reference_density();
            Some(Constrained::new(m_dot).unwrap())
        } else {
            None
        }
    }

    /// Sets the volumetric draw rate for a specific hour of the day.
    ///
    /// Accepts a value in units of flow (e.g., gallons per minute) and the
    /// corresponding hour index (0–23).
    ///
    /// Returns the updated model.
    fn set_draw_for_hour(mut self, draw: VolumeRate, hour_index: usize) -> Self {
        self.draw_schedule[hour_index] = draw;
        self
    }
}

impl Component for TanksInRoom<'_> {
    type Input = Input;
    type Output = Output;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let first_tank_input = self.prepare_first_tank_input(&input);
        let first_tank_output = self.first_tank.call(first_tank_input).unwrap();

        let second_tank_input = self.prepare_second_tank_input(&input);
        let second_tank_output = self.second_tank.call(second_tank_input).unwrap();

        Ok(Output {
            first_tank: first_tank_output,
            second_tank: second_tank_output,
        })
    }
}

/// A simulation of the `TanksInRoom` model using forward Euler integration.
#[derive(Debug)]
struct TanksInRoomSim<'a> {
    model: TanksInRoom<'a>,
}

impl<'a> Simulation for TanksInRoomSim<'a> {
    type Model = TanksInRoom<'a>;
    type StepError = Infallible;

    fn model(&self) -> &Self::Model {
        &self.model
    }

    fn advance_time(
        &self,
        state: &State<Self::Model>,
        dt: Duration,
    ) -> Result<<Self::Model as Component>::Input, Self::StepError> {
        let State { input, output } = state;
        let dt_time = dt.as_time();

        Ok(Input {
            time: input.time + dt_time,
            t_first_tank: input
                .t_first_tank
                .step(output.first_tank.state_derivative.temperature, dt_time),
            t_second_tank: input
                .t_second_tank
                .step(output.second_tank.state_derivative.temperature, dt_time),
            ..input.clone()
        })
    }
}

/// A convenience struct for collecting time series temperature data.
///
/// Stores temperature traces for ground water, room air, and both tanks,
/// formatted for plotting.
/// Each series contains `(time, temperature)` pairs where:
///
/// - Time is in hours
/// - Temperature is in celsius
#[derive(Debug, Default)]
struct PlotSeries {
    ground: Vec<[f64; 2]>,
    room: Vec<[f64; 2]>,
    first_tank: Vec<[f64; 2]>,
    second_tank: Vec<[f64; 2]>,
}

impl PlotSeries {
    /// Appends the current temperatures to each plot series.
    ///
    /// Extracts the time (in hours) and fluid temperatures (in °C) from the
    /// given simulation state and pushes them to the corresponding series.
    fn push(&mut self, state: &State<TanksInRoom>) {
        let push_temp = |vec: &mut Vec<[f64; 2]>, temp: ThermodynamicTemperature| {
            let time = state.input.time.get::<hour>();
            vec.push([time, temp.get::<degree_celsius>()]);
        };

        push_temp(&mut self.ground, state.input.t_ground);
        push_temp(&mut self.room, state.input.t_room);
        push_temp(&mut self.first_tank, state.input.t_first_tank);
        push_temp(&mut self.second_tank, state.input.t_second_tank);
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
            u_value: HeatTransfer::new::<watt_per_square_meter_kelvin>(0.5),
        },
    )
    .set_draw_for_hour(VolumeRate::new::<gallon_per_minute>(0.2), 11);

    // Create the simulation.
    let sim = TanksInRoomSim { model };

    // Specify the initial conditions.
    let initial_conditions = Input {
        time: Time::new::<second>(0.0),
        t_ground: ThermodynamicTemperature::new::<degree_celsius>(10.0),
        t_room: ThermodynamicTemperature::new::<degree_celsius>(20.0),
        t_first_tank: ThermodynamicTemperature::new::<degree_celsius>(70.0),
        t_second_tank: ThermodynamicTemperature::new::<degree_celsius>(50.0),
        q_dot_first_tank: Power::new::<watt>(10.0),
    };

    // Run the simulation with a 5 minute time step, storing values for plotting.
    let dt = Duration::from_secs(300);
    let mut series = PlotSeries::default();
    for step_result in sim.step_iter(initial_conditions, dt).take(2000) {
        let state = step_result.expect("Step should succeed");
        series.push(&state);
    }

    // Plot the temperatures.
    let app = PlotApp::new()
        .add_series("Ground Water Temp [°C]", &series.ground)
        .add_series("Room Temp [°C]", &series.room)
        .add_series("First Tank Temp [°C]", &series.first_tank)
        .add_series("Second Tank Temp [°C]", &series.second_tank);

    app.run("Tanks in Room Example").unwrap();
}
