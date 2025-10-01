//! Counter-flow effectiveness-NTU relationships.

use crate::thermal::hx::{
    capacitance_rate::CapacitanceRate,
    effectiveness_ntu::{
        Effectiveness, EffectivenessRelation, Ntu, NtuRelation, effectiveness_via, ntu_via,
    },
};

/// Counter-flow heat exchanger arrangement.
#[derive(Debug, Clone, Copy)]
pub struct CounterFlow;

impl EffectivenessRelation for CounterFlow {
    fn effectiveness(&self, ntu: Ntu, capacitance_rates: [CapacitanceRate; 2]) -> Effectiveness {
        effectiveness_via(ntu, capacitance_rates, |ntu, cr| {
            if cr < 1. {
                (1. - (-ntu * (1. - cr)).exp()) / (1. - cr * (-ntu * (1. - cr)).exp())
            } else {
                // cr == 1
                ntu / (1. + ntu)
            }
        })
    }
}

impl NtuRelation for CounterFlow {
    fn ntu(&self, effectiveness: Effectiveness, capacitance_rates: [CapacitanceRate; 2]) -> Ntu {
        ntu_via(effectiveness, capacitance_rates, |eff, cr| {
            if cr < 1. {
                (((1. - eff * cr) / (1. - eff)).ln()) / (1. - cr)
            } else {
                // cr == 1
                eff / (1. - eff)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use twine_core::constraint::ConstraintResult;
    use uom::si::{ratio::ratio, thermal_conductance::watt_per_kelvin};

    use super::*;

    #[test]
    fn roundtrip() -> ConstraintResult<()> {
        let ntus = [0., 0.1, 0.5, 1., 5.];
        let capacitance_rates = [
            // c_r == 0
            [1., f64::INFINITY],
            // c_r == 0.25
            [1., 4.],
            // c_r == 0.5
            [1., 2.],
            // c_r == 1
            [1., 1.],
        ];

        for ntu in ntus {
            for pair in capacitance_rates {
                let rates = [
                    CapacitanceRate::new::<watt_per_kelvin>(pair[0])?,
                    CapacitanceRate::new::<watt_per_kelvin>(pair[1])?,
                ];

                let eff = CounterFlow.effectiveness(Ntu::new(ntu)?, rates);
                let back = CounterFlow.ntu(eff, rates);

                assert_relative_eq!(back.get::<ratio>(), ntu, max_relative = 1e-12);
            }
        }

        Ok(())
    }
}
