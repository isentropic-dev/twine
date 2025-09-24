use uom::si::ratio::ratio;

use crate::thermal::hx::{
    CapacityRatio,
    effectiveness_ntu::{Effectiveness, EffectivenessNtu, Ntu},
};

/// A counter-flow arrangement.
#[derive(Debug, Clone, Copy)]
pub struct CounterFlow;

impl EffectivenessNtu for CounterFlow {
    fn effectiveness(&self, ntu: Ntu, capacity_ratio: CapacityRatio) -> Effectiveness {
        let cr = capacity_ratio.get::<ratio>();

        if cr == 0. {
            return Effectiveness::infinite_capacitance_rate(ntu);
        }

        Effectiveness::new({
            let ntu = ntu.get::<ratio>();
            if cr < 1. {
                (1. - (-ntu * (1. - cr)).exp()) / (1. - cr * (-ntu * (1. - cr)).exp())
            } else {
                ntu / (1. + ntu)
            }
        })
        .expect("ntu should always yield valid effectiveness")
    }

    fn ntu(&self, effectiveness: Effectiveness, capacity_ratio: CapacityRatio) -> Ntu {
        let cr = capacity_ratio.get::<ratio>();

        if cr == 0. {
            return Ntu::infinite_capacitance_rate(effectiveness);
        }

        Ntu::new({
            let eff = effectiveness.get::<ratio>();
            if cr < 1. {
                (((1. - eff * cr) / (1. - eff)).ln()) / (1. - cr)
            } else {
                // cr == 1
                eff / (1. - eff)
            }
        })
        .expect("effectiveness should always yield valid ntu")
    }
}
