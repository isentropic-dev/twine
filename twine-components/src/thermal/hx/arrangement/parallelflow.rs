//! Parallel-flow effectiveness-NTU relationships.

use crate::thermal::hx::{
    Effectiveness, EffectivenessNtu, Ntu,
    effectiveness_ntu::{effectiveness_via, ntu_via},
};

/// Parallel-flow heat exchanger arrangement.
#[derive(Debug, Clone, Copy)]
pub struct ParallelFlow;

impl EffectivenessNtu for ParallelFlow {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacity_ratio: crate::thermal::hx::CapacityRatio,
    ) -> Effectiveness {
        effectiveness_via(ntu, capacity_ratio, |ntu, cr| {
            (1. - (-ntu * (1. + cr)).exp()) / (1. + cr)
        })
    }

    fn ntu(
        &self,
        effectiveness: crate::thermal::hx::Effectiveness,
        capacity_ratio: crate::thermal::hx::CapacityRatio,
    ) -> Ntu {
        ntu_via(effectiveness, capacity_ratio, |eff, cr| {
            -(1. - eff * (1. + cr)).ln() / (1. + cr)
        })
    }
}
