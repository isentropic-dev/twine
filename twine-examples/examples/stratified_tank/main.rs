//! # Stratified Tank Simulation
//!
//! To run this example:
//!
//! ```sh
//! cargo run --example stratified_tank --release
//! ```

mod model;
mod plotting;
mod simulation;

use std::time::Duration;

use jiff::civil::{DateTime, Time};
use twine_components::{
    controller::SwitchState,
    schedule::step_schedule::StepSchedule,
    thermal::stratified_tank::{
        Fluid, Geometry, Insulation, Location, PortLocation, StratifiedTank,
    },
};
use twine_core::Simulation;
use twine_plot::PlotApp;
use uom::si::{
    f64::{
        Length, MassDensity, SpecificHeatCapacity, ThermalConductivity, ThermodynamicTemperature,
        VolumeRate,
    },
    length::meter,
    mass_density::kilogram_per_cubic_meter,
    specific_heat_capacity::kilojoule_per_kilogram_kelvin,
    thermal_conductivity::watt_per_meter_kelvin,
    thermodynamic_temperature::degree_celsius,
    volume_rate::gallon_per_minute,
};

use model::{ModelInput, TankModel};
use simulation::TankSimulation;

/// Number of nodes in the tank.
const NODES: usize = 20;

/// Node containing the element.
const ELEMENT_LOCATION: usize = 12;

/// Rated power of the electric heating element, in kW.
const ELEMENT_KW: f64 = 6.0;

/// Thermostat temperature setpoint for the element, in °C.
const SETPOINT_C: f64 = 50.0;

/// Thermostat deadband width, in °C.
const DEADBAND_C: f64 = 10.0;

/// Days to simulate.
const DAYS: usize = 5;

fn configure_model() -> TankModel<NODES> {
    // Configure the tank.
    let tank = StratifiedTank::new::<NODES>(
        // Use typical values for water.
        Fluid {
            density: MassDensity::new::<kilogram_per_cubic_meter>(990.0),
            specific_heat: SpecificHeatCapacity::new::<kilojoule_per_kilogram_kelvin>(4.18),
            thermal_conductivity: ThermalConductivity::new::<watt_per_meter_kelvin>(0.6),
        },
        // Model a 120 gallon tank.
        Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(0.6),
            height: Length::new::<meter>(1.6),
        },
        // Assume perfect insulation.
        Insulation::Adiabatic,
        // Heating element is located in a single node.
        [Location::point_in_node(ELEMENT_LOCATION)],
        // Water flows through the tank from bottom to top.
        [PortLocation {
            inlet: Location::tank_bottom(),
            outlet: Location::tank_top(),
        }],
    )
    .unwrap();

    // Configure the draw schedule.
    let daily_draw_schedule = StepSchedule::new([
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
    .unwrap();

    TankModel {
        tank,
        daily_draw_schedule,
    }
}

fn main() {
    // Initialize the model and simulation.
    let model = configure_model();
    let sim = TankSimulation::new(model);

    // Specify the initial conditions.
    let initial_conditions = ModelInput {
        datetime: DateTime::default(),
        t_ground: ThermodynamicTemperature::new::<degree_celsius>(10.0),
        t_room: ThermodynamicTemperature::new::<degree_celsius>(20.0),
        t_tank: [ThermodynamicTemperature::new::<degree_celsius>(20.0); NODES],
        element_state: SwitchState::Off,
    };

    // Run the simulation for `DAYS` with a one minute time step.
    let mut series = plotting::PlotSeries::default();
    for step_result in sim
        .into_step_iter(initial_conditions, Duration::from_secs(60))
        .take(DAYS * 24 * 60)
    {
        let state = step_result.expect("Step should succeed");
        series.push(&state);
    }

    // Plot some results.
    let app = PlotApp::new()
        .add_series("Ground Water Temp [°C]", &series.ground)
        .add_series("Tank Top Temp [°C]", &series.tank_top)
        .add_series("Tank Middle Temp [°C]", &series.tank_middle)
        .add_series("Tank Bottom Temp [°C]", &series.tank_bottom)
        .add_series("Draw [GPM]", &series.draw)
        .add_series("Element [kW]", &series.element);

    app.run("Stratified Tank Example").unwrap();
}
