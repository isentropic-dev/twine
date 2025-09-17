use twine_core::constraint::{Constrained, ConstraintResult, NonNegative, UnitInterval};
use uom::si::{f64::Ratio, ratio::ratio};

type Effectiveness = Constrained<Ratio, UnitInterval>;
type Ntu = Constrained<Ratio, NonNegative>;
type CapacityRatio = Constrained<Ratio, UnitInterval>;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum EffectivenessArrangement {
    OneFluid {
        ntu: Ntu,
    },
    CounterFlow {
        ntu: Ntu,
        capacity_ratio: CapacityRatio,
    },
}

impl EffectivenessArrangement {
    pub fn one_fluid(ntu: Ratio) -> ConstraintResult<Self> {
        Ok(Self::OneFluid {
            ntu: NonNegative::new(ntu)?,
        })
    }

    pub fn counter_flow(ntu: Ratio, capacity_ratio: Ratio) -> ConstraintResult<Self> {
        Ok(Self::CounterFlow {
            ntu: NonNegative::new(ntu)?,
            capacity_ratio: UnitInterval::new(capacity_ratio)?,
        })
    }
}

pub fn effectiveness(arrangement: EffectivenessArrangement) -> ConstraintResult<Effectiveness> {
    let effectiveness = match arrangement {
        EffectivenessArrangement::OneFluid { ntu } => {
            let ntu = ntu.into_inner().get::<ratio>();

            1. - (-ntu).exp()
        }
        EffectivenessArrangement::CounterFlow {
            ntu,
            capacity_ratio,
        } => {
            let ntu = ntu.into_inner().get::<ratio>();
            let cr = capacity_ratio.into_inner().get::<ratio>();

            if cr < 1. {
                (1. - (-ntu * (1. - cr)).exp()) / (1. - cr * (-ntu * (1. - cr)).exp())
            } else {
                // cr == 1
                ntu / (1. + ntu)
            }
        }
    };

    UnitInterval::new(Ratio::new::<ratio>(effectiveness))
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum NtuArrangement {
    OneFluid {
        effectiveness: Effectiveness,
    },
    CounterFlow {
        effectiveness: Effectiveness,
        capacity_ratio: CapacityRatio,
    },
}

impl NtuArrangement {
    pub fn one_fluid(effectiveness: Ratio) -> ConstraintResult<Self> {
        Ok(Self::OneFluid {
            effectiveness: UnitInterval::new(effectiveness)?,
        })
    }

    pub fn counter_flow(effectiveness: Ratio, capacity_ratio: Ratio) -> ConstraintResult<Self> {
        Ok(Self::CounterFlow {
            effectiveness: UnitInterval::new(effectiveness)?,
            capacity_ratio: UnitInterval::new(capacity_ratio)?,
        })
    }
}

pub fn ntu(arrangement: NtuArrangement) -> ConstraintResult<Ntu> {
    let ntu = match arrangement {
        NtuArrangement::OneFluid { effectiveness } => {
            let eff = effectiveness.into_inner().get::<ratio>();
            -(1. - eff).ln()
        }
        NtuArrangement::CounterFlow {
            effectiveness,
            capacity_ratio,
        } => {
            let eff = effectiveness.into_inner().get::<ratio>();
            let cr = capacity_ratio.into_inner().get::<ratio>();

            if cr < 1. {
                (((1. - eff * cr) / (1. - eff)).ln()) / (1. - cr)
            } else {
                // cr == 1
                eff / (1. - eff)
            }
        }
    };

    NonNegative::new(Ratio::new::<ratio>(ntu))
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn roundtrip_one_fluid() -> ConstraintResult<()> {
        for ntu_val in [0., 0.1, 0.5, 1., 5.] {
            let eff = effectiveness(EffectivenessArrangement::one_fluid(Ratio::new::<ratio>(
                ntu_val,
            ))?)?;
            let back = ntu(NtuArrangement::one_fluid(eff.into_inner())?)?;

            assert_relative_eq!(back.into_inner().get::<ratio>(), ntu_val);
        }
        Ok(())
    }

    #[test]
    fn roundtrip_counter_flow() -> ConstraintResult<()> {
        let ntu_vals = [0., 0.1, 0.5, 1., 5.];
        let cr_vals = [0., 0.25, 0.5, 1.];

        for ntu_val in ntu_vals {
            for cr_val in cr_vals {
                let cr_ratio = Ratio::new::<ratio>(cr_val);

                let eff = effectiveness(EffectivenessArrangement::counter_flow(
                    Ratio::new::<ratio>(ntu_val),
                    cr_ratio,
                )?)?;
                let back = ntu(NtuArrangement::counter_flow(eff.into_inner(), cr_ratio)?)?;

                assert_relative_eq!(
                    back.into_inner().get::<ratio>(),
                    ntu_val,
                    max_relative = 1e-12
                );
            }
        }
        Ok(())
    }
}
