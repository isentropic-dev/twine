use uom::si::f64::ThermalConductance;

use crate::thermal::hx::StreamInlet;

pub(super) mod known_conductance_and_inlets;

#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    KnownConductanceAndInlets {
        ua: ThermalConductance,
        inlets: [StreamInlet; 2],
    },
}
