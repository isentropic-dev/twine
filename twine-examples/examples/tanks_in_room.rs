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

use std::{convert::Infallible, ops::Div, time::Duration};

use twine_components::{
    fluid::IncompressibleLiquid,
    integrators::ForwardEuler,
    thermal::tank::{Tank, TankConfig, TankInput, TankOutput},
};
use twine_core::{
    thermo::units::PositiveMassRate, Component, Integrator, Simulation, State, TimeDerivativeOf,
    TimeIntegrable,
};
use twine_plot::PlotApp;
use uom::{
    si::{
        area::square_foot,
        f64::{Area, HeatTransfer, Power, ThermodynamicTemperature, Time, Volume, VolumeRate},
        heat_transfer::watt_per_square_meter_kelvin,
        power::watt,
        thermodynamic_temperature::degree_celsius,
        time::{hour, second},
        volume::gallon,
        volume_rate::gallon_per_minute,
    },
    ConstZero,
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
struct TanksInRoom {
    fluid: IncompressibleLiquid,
    first_tank: Tank<IncompressibleLiquid>,
    second_tank: Tank<IncompressibleLiquid>,
    draw_schedule: [VolumeRate; 24],
}

impl TanksInRoom {
    /// Creates a new two-tank system using the provided fluid and tank configurations.
    ///
    /// The two tanks are connected in series and share the same fluid model.
    fn new(
        fluid: IncompressibleLiquid,
        first_tank_config: TankConfig,
        second_tank_config: TankConfig,
    ) -> Self {
        Self {
            fluid,
            first_tank: Tank::new(fluid, first_tank_config),
            second_tank: Tank::new(fluid, second_tank_config),
            draw_schedule: [VolumeRate::ZERO; 24],
        }
    }

    /// Prepares the input for the first tank based on current simulation state.
    fn prepare_first_tank_input(&self, input: &Input) -> TankInput<IncompressibleLiquid> {
        TankInput {
            ambient_temperature: input.t_room,
            heat_input: input.q_dot_first_tank,
            inlet_state: input.t_ground,
            mass_flow_rate: self.draw_at_time(input.time),
            tank_state: input.t_first_tank,
        }
    }

    /// Prepares the input for the second tank based on current simulation state.
    fn prepare_second_tank_input(&self, input: &Input) -> TankInput<IncompressibleLiquid> {
        TankInput {
            ambient_temperature: input.t_room,
            heat_input: Power::ZERO,
            inlet_state: input.t_first_tank,
            mass_flow_rate: self.draw_at_time(input.time),
            tank_state: input.t_second_tank,
        }
    }

    /// Returns the mass flow rate through the tanks at a given time.
    ///
    /// Flow is looked up from the draw schedule using the current hour of day,
    /// then converted from volumetric to mass flow using the fluid's density.
    fn draw_at_time(&self, time: Time) -> PositiveMassRate {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let hour_of_day = time.get::<hour>().floor() as usize % 24;

        let draw = self.draw_schedule[hour_of_day];

        (draw * self.fluid.density)
            .try_into()
            .expect("Draw cannot be negative")
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
    first_tank: TankOutput,
    second_tank: TankOutput,
}

impl Component for TanksInRoom {
    type Input = Input;
    type Output = Output;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let first_tank_input = self.prepare_first_tank_input(&input);
        let first_tank_output = self
            .first_tank
            .call(first_tank_input)
            .expect("Call cannot fail with the `IncompressibleLiquid` model");

        let second_tank_input = self.prepare_second_tank_input(&input);
        let second_tank_output = self
            .second_tank
            .call(second_tank_input)
            .expect("Call cannot fail with the `IncompressibleLiquid` model");

        Ok(Output {
            first_tank: first_tank_output,
            second_tank: second_tank_output,
        })
    }
}

/// A simulation of the `TanksInRoom` model using a `ForwardEuler` integrator.
#[derive(Debug)]
struct TanksInRoomSim {
    model: TanksInRoom,
    integrator: ForwardEuler<StateVariables>,
}

impl Simulation for TanksInRoomSim {
    type Model = TanksInRoom;
    type Integrator = ForwardEuler<StateVariables>;

    fn model(&self) -> &Self::Model {
        &self.model
    }

    fn integrator(&self) -> &Self::Integrator {
        &self.integrator
    }

    fn prepare_integrator_input(
        &self,
        state: &State<Self::Model>,
    ) -> <Self::Integrator as Integrator>::Input {
        let state_vars = StateVariables {
            t_second_tank: state.input.t_second_tank,
            t_first_tank: state.input.t_first_tank,
        };

        let state_derivs = StateVariableDerivatives {
            t_second_tank_dt: state.output.second_tank.tank_temperature_derivative,
            t_first_tank_dt: state.output.first_tank.tank_temperature_derivative,
        };

        (state_vars, state_derivs)
    }

    fn prepare_model_input(
        &self,
        prev_state: &State<Self::Model>,
        integrator_output: <Self::Integrator as Integrator>::Output,
        actual_dt: std::time::Duration,
    ) -> <Self::Model as Component>::Input {
        let dt_time = Time::new::<second>(actual_dt.as_secs_f64());
        Input {
            time: prev_state.input.time + dt_time,
            t_second_tank: integrator_output.t_second_tank,
            t_first_tank: integrator_output.t_first_tank,
            ..prev_state.input
        }
    }
}

/// State variables for the simulation.
#[derive(Clone, Debug)]
struct StateVariables {
    t_first_tank: ThermodynamicTemperature,
    t_second_tank: ThermodynamicTemperature,
}

/// Time derivatives of the state variables.
#[derive(Clone, Debug)]
struct StateVariableDerivatives {
    t_first_tank_dt: TimeDerivativeOf<ThermodynamicTemperature>,
    t_second_tank_dt: TimeDerivativeOf<ThermodynamicTemperature>,
}

impl Div<Time> for StateVariables {
    type Output = StateVariableDerivatives;

    fn div(self, rhs: Time) -> Self::Output {
        Self::Output {
            t_first_tank_dt: self.t_first_tank / rhs,
            t_second_tank_dt: self.t_second_tank / rhs,
        }
    }
}

impl TimeIntegrable for StateVariables {
    fn step_by_time(self, derivative: StateVariableDerivatives, dt: Time) -> Self {
        Self {
            t_first_tank: self.t_first_tank + derivative.t_first_tank_dt * dt,
            t_second_tank: self.t_second_tank + derivative.t_second_tank_dt * dt,
        }
    }
}

/// A convenience struct for collecting time series temperature data.
///
/// Stores temperature traces for ground water, room air, and both tanks,
/// formatted for plotting.
/// Each series contains `(time, temperature)` pairs where:
///
/// - Time is in hours.
/// - Temperature is in degrees Celsius.
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
        IncompressibleLiquid::water(),
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
    let sim = TanksInRoomSim {
        model,
        integrator: ForwardEuler::new(),
    };

    // Specify the initial conditions.
    let initial_conditions = Input {
        time: Time::new::<second>(0.0),
        t_ground: ThermodynamicTemperature::new::<degree_celsius>(10.0),
        t_room: ThermodynamicTemperature::new::<degree_celsius>(20.0),
        t_second_tank: ThermodynamicTemperature::new::<degree_celsius>(50.0),
        t_first_tank: ThermodynamicTemperature::new::<degree_celsius>(70.0),
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
