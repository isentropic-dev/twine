use jiff::civil::DateTime;
use twine_components::controller::SwitchState;
use twine_core::State;
use uom::si::{thermodynamic_temperature::degree_celsius, volume_rate::gallon_per_minute};

use super::{ELEMENT_KW, TankModel};

/// A convenience struct for collecting time series data.
///
/// Each series contains `(time, value)` pairs where:
///
/// - Time is in hours
/// - Temperature is in °C
/// - Draw is in GPM
/// - Element heating is in kW
#[derive(Debug, Default)]
pub(super) struct PlotSeries {
    pub(super) ground: Vec<[f64; 2]>,
    pub(super) room: Vec<[f64; 2]>,
    pub(super) tank_top: Vec<[f64; 2]>,
    pub(super) tank_middle: Vec<[f64; 2]>,
    pub(super) tank_bottom: Vec<[f64; 2]>,
    pub(super) draw: Vec<[f64; 2]>,
    pub(super) element: Vec<[f64; 2]>,
}

impl PlotSeries {
    /// Appends the current values to each plot series.
    pub(super) fn push<const N: usize>(&mut self, State { input, output }: &State<TankModel<N>>) {
        let elapsed_hr = input
            .datetime
            .duration_since(DateTime::default())
            .as_secs_f64()
            / 3600.0;

        self.ground
            .push([elapsed_hr, input.t_ground.get::<degree_celsius>()]);

        self.room
            .push([elapsed_hr, input.t_room.get::<degree_celsius>()]);

        self.tank_bottom.push([
            elapsed_hr,
            output.tank.temperatures[0].get::<degree_celsius>(),
        ]);

        self.tank_middle.push([
            elapsed_hr,
            output.tank.temperatures[N / 2].get::<degree_celsius>(),
        ]);

        self.tank_top.push([
            elapsed_hr,
            output.tank.temperatures[N - 1].get::<degree_celsius>(),
        ]);

        self.draw
            .push([elapsed_hr, output.draw.get::<gallon_per_minute>()]);

        self.element.push([
            elapsed_hr,
            match output.element_state {
                SwitchState::Off => 0.0,
                SwitchState::On => ELEMENT_KW,
            },
        ]);
    }
}
