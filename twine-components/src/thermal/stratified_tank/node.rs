use uom::si::f64::{ThermalConductance, Volume};

use super::Adjacent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Node<const P: usize, const Q: usize> {
    pub(super) vol: Volume,
    pub(super) ua: Adjacent<ThermalConductance>,
    pub(super) aux_heat_weights: [f64; Q],
    pub(super) port_inlet_weights: [f64; P],
    pub(super) port_outlet_weights: [f64; P],
}
