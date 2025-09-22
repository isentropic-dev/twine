use crate::thermal::hx::{capacity_ratio::CapacityRatio, effectiveness::Effectiveness, ntu::Ntu};

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum Arrangement {
    CounterFlow,
}

impl Arrangement {
    pub fn effectiveness(&self, ntu: Ntu, capacity_ratio: CapacityRatio) -> Effectiveness {
        match self {
            Arrangement::CounterFlow => Effectiveness::counter_flow(ntu, capacity_ratio),
        }
    }

    pub fn ntu(&self, effectiveness: Effectiveness, capacity_ratio: CapacityRatio) -> Ntu {
        match self {
            Arrangement::CounterFlow => Ntu::counter_flow(effectiveness, capacity_ratio),
        }
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
        let arrangements = [Arrangement::CounterFlow];
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
