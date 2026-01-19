//! Results types for discretized heat exchanger solving.

use twine_thermo::State;
use uom::si::f64::{TemperatureInterval, ThermalConductance};

use super::HeatTransferRate;

/// Node states and performance metrics for a discretized heat exchanger.
///
/// Node arrays follow the physical layout from left (0) to right (N-1).
/// The top stream always flows from node 0 to node N-1.
/// The bottom stream flows from node 0 to node N-1 for parallel flow and from
/// node N-1 to node 0 for counterflow.
#[derive(Debug, Clone)]
pub struct Results<TopFluid, BottomFluid, const N: usize> {
    /// Top stream node states, ordered from left (0) to right (N-1).
    pub top: [State<TopFluid>; N],

    /// Bottom stream node states, ordered from left (0) to right (N-1).
    pub bottom: [State<BottomFluid>; N],

    /// Heat transfer rate.
    pub q_dot: HeatTransferRate,

    /// Total heat exchanger conductance.
    pub ua: ThermalConductance,

    /// Minimum hot-to-cold temperature difference and its node.
    pub min_delta_t: MinDeltaT,
}

/// Minimum hot-to-cold temperature difference and its node index.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinDeltaT {
    /// Minimum hot-to-cold temperature difference.
    ///
    /// For [`HeatTransferRate::None`], this is the minimum absolute temperature
    /// difference between the streams.
    pub value: TemperatureInterval,

    /// Node index where the minimum temperature difference occurs.
    pub node: usize,
}
