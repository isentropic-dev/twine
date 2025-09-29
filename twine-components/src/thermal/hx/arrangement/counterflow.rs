//! Counter-flow effectiveness-NTU relationships.

use crate::thermal::hx::{
    CapacityRatio,
    effectiveness_ntu::{Effectiveness, EffectivenessNtu, Ntu, effectiveness_via, ntu_via},
};

/// Counter-flow heat exchanger arrangement.
#[derive(Debug, Clone, Copy)]
pub struct CounterFlow;

impl EffectivenessNtu for CounterFlow {
    fn effectiveness(&self, ntu: Ntu, capacity_ratio: CapacityRatio) -> Effectiveness {
        effectiveness_via(ntu, capacity_ratio, |ntu, cr| {
            if cr < 1. {
                (1. - (-ntu * (1. - cr)).exp()) / (1. - cr * (-ntu * (1. - cr)).exp())
            } else {
                ntu / (1. + ntu)
            }
        })
    }

    fn ntu(&self, effectiveness: Effectiveness, capacity_ratio: CapacityRatio) -> Ntu {
        ntu_via(effectiveness, capacity_ratio, |eff, cr| {
            if cr < 1. {
                (((1. - eff * cr) / (1. - eff)).ln()) / (1. - cr)
            } else {
                // cr == 1
                eff / (1. - eff)
            }
        })
    }
}
