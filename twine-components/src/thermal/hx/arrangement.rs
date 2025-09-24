use uom::si::ratio::ratio;

use crate::thermal::hx::{capacity_ratio::CapacityRatio, effectiveness::Effectiveness, ntu::Ntu};

pub trait Arrangement {
    /// Calculate the effectiveness for an arrangement given the [NTU](Ntu) and
    ///[capacity ratio](CapacityRatio).
    fn effectiveness(&self, ntu: Ntu, capacity_ratio: CapacityRatio) -> Effectiveness;

    /// Calculate the [NTU](Ntu) for an arrangement given the
    /// [effectiveness](Effectiveness) and [capacity ratio](CapacityRatio).
    fn ntu(&self, effectiveness: Effectiveness, capacity_ratio: CapacityRatio) -> Ntu;
}

/// A counter-flow arrangement.
#[derive(Debug, Clone, Copy)]
pub struct CounterFlow;

impl Arrangement for CounterFlow {
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use twine_core::constraint::ConstraintResult;
    use uom::si::ratio::ratio;

    #[test]
    fn roundtrips() -> ConstraintResult<()> {
        let arrangements = vec![CounterFlow];
        let ntus = [0., 0.1, 0.5, 1., 5.];
        let capacity_ratios = [0., 0.25, 0.5, 1.];

        for arrangement in arrangements {
            for ntu in ntus {
                for capacity_ratio in capacity_ratios {
                    let cr = CapacityRatio::new(capacity_ratio)?;

                    let eff = arrangement.effectiveness(Ntu::new(ntu)?, cr);
                    let back = arrangement.ntu(eff, cr);

                    assert_relative_eq!(back.get::<ratio>(), ntu, max_relative = 1e-12);
                }
            }
        }

        Ok(())
    }
}
