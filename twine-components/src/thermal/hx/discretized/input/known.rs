use crate::thermal::hx::discretized::{Inlets, MassFlows, PressureDrops};

/// Core inputs for a discretized heat exchanger.
///
/// Combined with a [`Given`] constraint, these inputs define the boundary
/// problem without requiring thermodynamic property evaluation.
#[derive(Debug, Clone)]
pub struct Known<TopFluid, BottomFluid> {
    /// Inlet states for the two streams.
    pub inlets: Inlets<TopFluid, BottomFluid>,

    /// Mass flow rates for the two streams.
    pub m_dot: MassFlows,

    /// Total pressure drops for the two streams.
    pub dp: PressureDrops,
}
