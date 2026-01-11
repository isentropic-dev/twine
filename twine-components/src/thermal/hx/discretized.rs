//! Discretized counter-flow and parallel-flow heat exchanger modeling.
//!
//! A discretized heat exchanger divides the flow into a linear series of
//! constant-property sub-exchangers so thermodynamic properties can vary
//! along a linear array of nodes, supporting real-fluid behavior.

mod heat_transfer_rate;
mod input;
mod solve;

pub use heat_transfer_rate::HeatTransferRate;
pub use input::{Given, Inlets, Known, MassFlows, PressureDrops};
pub use solve::{DiscretizedHx, Results, SolveError};
