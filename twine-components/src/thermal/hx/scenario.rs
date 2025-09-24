use uom::si::f64::ThermalConductance;

use crate::thermal::hx::StreamInlet;

pub(super) mod known_conductance_and_inlets;

/// Heat exchanger analysis scenarios.
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    /// Known heat exchanger conductance and inlet conditions.
    KnownConductanceAndInlets {
        /// Heat exchanger conductance.
        ua: ThermalConductance,
        /// Inlet conditions.
        inlets: [StreamInlet; 2],
    },
}
