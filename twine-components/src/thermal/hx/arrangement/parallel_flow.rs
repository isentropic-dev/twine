//! Parallel-flow effectiveness-NTU relationships.

use crate::thermal::hx::{
    capacity_ratio::CapacityRatio,
    effectiveness_ntu::{
        Effectiveness, EffectivenessRelation, Ntu, NtuRelation, effectiveness_via, ntu_via,
    },
};

/// Parallel-flow heat exchanger arrangement.
#[derive(Debug, Clone, Copy)]
pub struct ParallelFlow;

impl EffectivenessRelation for ParallelFlow {
    fn effectiveness(&self, ntu: Ntu, capacity_ratio: CapacityRatio) -> Effectiveness {
        effectiveness_via(ntu, capacity_ratio, |ntu, cr| {
            (1. - (-ntu * (1. + cr)).exp()) / (1. + cr)
        })
    }
}

impl NtuRelation for ParallelFlow {
    fn ntu(&self, effectiveness: Effectiveness, capacity_ratio: CapacityRatio) -> Ntu {
        ntu_via(effectiveness, capacity_ratio, |eff, cr| {
            -(1. - eff * (1. + cr)).ln() / (1. + cr)
        })
    }
}
