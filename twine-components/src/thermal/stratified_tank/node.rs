use uom::si::f64::ThermalConductance;

use super::{InverseHeatCapacity, InverseVolume};

/// Node configuration (fixed at tank creation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Node<const P: usize, const Q: usize> {
    pub(super) inv_volume: InverseVolume,
    pub(super) inv_heat_capacity: InverseHeatCapacity,
    pub(super) ua: Adjacent<ThermalConductance>,
    pub(super) aux_heat_weights: [f64; Q],
    pub(super) port_inlet_weights: [f64; P],
    pub(super) port_outlet_weights: [f64; P],
}

/// Values at the bottom, side, and top faces of a node.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Adjacent<T> {
    pub(super) bottom: T,
    pub(super) side: T,
    pub(super) top: T,
}
