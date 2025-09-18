//! Effectiveness–NTU relations for idealized heat exchanger configurations.

use twine_core::constraint::{Constrained, ConstraintResult, NonNegative, UnitInterval};
use uom::si::{f64::Ratio, ratio::ratio};

/// Heat exchanger effectiveness.
type Effectiveness = Constrained<Ratio, UnitInterval>;
/// Number of transfer units.
type Ntu = Constrained<Ratio, NonNegative>;
/// Capacity ratio, defined as `C_min / C_max` and limited to `[0, 1]`.
type CapacityRatio = Constrained<Ratio, UnitInterval>;

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
/// Input data for computing heat exchanger effectiveness from NTU.
///
/// Use the associated constructor to validate raw ratios before invoking
/// [`effectiveness`].
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
    /// Creates effectiveness input for a heat exchanger where the capacitance
    /// rate of one stream is much greater than the other (i.e. the capacity
    /// ratio == 0).
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if:
    ///
    /// - if `ntu` is negative.
    pub fn one_fluid(ntu: Ratio) -> ConstraintResult<Self> {
        Ok(Self::OneFluid {
            ntu: NonNegative::new(ntu)?,
        })
    }

    /// Creates effectiveness input for a counter-flow heat exchanger.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if:
    ///
    /// - if `ntu` is negative.
    /// - if `capacity_ratio` is not within `[0, 1]`.
    pub fn counter_flow(ntu: Ratio, capacity_ratio: Ratio) -> ConstraintResult<Self> {
        Ok(Self::CounterFlow {
            ntu: NonNegative::new(ntu)?,
            capacity_ratio: UnitInterval::new(capacity_ratio)?,
        })
    }
}

/// Computes heat exchanger effectiveness.
///
/// # Errors
///
/// Returns [`ConstraintError`] if the supplied parameters fall outside their
/// valid ranges. This is unlikely if the respective constructors were used to
/// create the [`EffectivenessArrangement`].
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
/// Input data for computing NTU from heat exchanger effectiveness.
///
/// Use the provided constructors to validate raw ratios before calling
/// [`ntu`].
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
    /// Creates NTU input for a heat exchanger where the capacitance rate of one
    /// stream is much greater than the other (i.e. the capacity ratio == 0).
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if:
    ///
    /// - `effectiveness` is outsite `[0, 1]`.
    pub fn one_fluid(effectiveness: Ratio) -> ConstraintResult<Self> {
        Ok(Self::OneFluid {
            effectiveness: UnitInterval::new(effectiveness)?,
        })
    }

    /// Creates NTU input for a counter-flow heat exchanger.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if:
    ///
    /// - `effectiveness` is outside `[0, 1]`.
    /// - `capacity_ratio` is outside `[0, 1]`.
    pub fn counter_flow(effectiveness: Ratio, capacity_ratio: Ratio) -> ConstraintResult<Self> {
        Ok(Self::CounterFlow {
            effectiveness: UnitInterval::new(effectiveness)?,
            capacity_ratio: UnitInterval::new(capacity_ratio)?,
        })
    }
}

/// Computes NTU for a heat exchanger.
///
/// # Errors
///
/// Returns [`ConstraintError`] when the provided values violate the expected
/// ranges. This should not happen when values are constructed through
/// [`NtuArrangement`].
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
