use std::marker::PhantomData;

use crate::thermal::hx::{
    CapacitanceRate, Effectiveness, Ntu,
    effectiveness_ntu::{EffectivenessRelation, NtuRelation, effectiveness_via, ntu_via},
};

/// Shell-and-tube heat exchanger arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellAndTube<const S: u16, const T: u16> {
    _marker: PhantomData<()>,
}

impl<const S: u16, const T: u16> ShellAndTube<S, T> {
    const fn validate() -> Result<(), ShellAndTubeConfigError> {
        if S == 0 {
            return Err(ShellAndTubeConfigError::ZeroShellPasses);
        }
        if S > u16::MAX / 2 {
            return Err(ShellAndTubeConfigError::ShellPassOverflow);
        }
        if T < 2 * S {
            return Err(ShellAndTubeConfigError::InsufficientTubePasses);
        }
        if !T.is_multiple_of(2 * S) {
            return Err(ShellAndTubeConfigError::TubePassesNotMultiple);
        }
        Ok(())
    }

    /// Construct a validated shell-and-tube arrangement configuration.
    ///
    /// # Errors
    ///
    /// Returns [`ShellAndTubeConfigError`] when the pass counts violate the supported
    /// shell-and-tube families (one shell pass with any even number of tube passes, or
    /// `N` shell passes with a tube pass count that is an even multiple of `N`).
    pub const fn new() -> Result<Self, ShellAndTubeConfigError> {
        if let Err(err) = Self::validate() {
            return Err(err);
        }

        Ok(Self {
            _marker: PhantomData,
        })
    }
}

impl<const S: u16, const T: u16> EffectivenessRelation for ShellAndTube<S, T> {
    fn effectiveness(&self, ntu: Ntu, capacitance_rates: [CapacitanceRate; 2]) -> Effectiveness {
        let eff_1: fn(f64, f64) -> f64 = |ntu_1, cr| {
            2. / (1.
                + cr
                + (1. + cr.powi(2)).sqrt() * (1. + (-ntu_1 * (1. + cr.powi(2)).sqrt()).exp())
                    / (1. - (-ntu_1 * (1. + cr.powi(2)).sqrt()).exp()))
        };

        if S == 1 {
            effectiveness_via(ntu, capacitance_rates, eff_1)
        } else {
            effectiveness_via(ntu, capacitance_rates, |ntu_1, cr| {
                let eff_1 = eff_1(ntu_1, cr);

                if cr < 1. {
                    (((1. - eff_1 * cr) / (1. - eff_1)).powi(S.into()) - 1.)
                        / (((1. - eff_1 * cr) / (1. - eff_1)).powi(S.into()) - cr)
                } else {
                    // cr == 1
                    (f64::from(S) * eff_1) / (1. + eff_1 * (f64::from(S) - 1.))
                }
            })
        }
    }
}

impl<const S: u16, const T: u16> NtuRelation for ShellAndTube<S, T> {
    fn ntu(&self, effectiveness: Effectiveness, capacitance_rates: [CapacitanceRate; 2]) -> Ntu {
        let ntu_1: fn(f64, f64) -> f64 = |eff_1, cr| {
            let e = (2. - eff_1 * (1. + cr)) / (eff_1 * (1. + cr.powi(2)).sqrt());
            ((e + 1.) / (e - 1.)).ln() / (1. + cr.powi(2)).sqrt()
        };

        if S == 1 {
            ntu_via(effectiveness, capacitance_rates, ntu_1)
        } else {
            ntu_via(effectiveness, capacitance_rates, |eff, cr| {
                let eff_1 = if cr < 1. {
                    let f = ((eff * cr - 1.) / (eff - 1.)).powf(1.0 / f64::from(S));
                    (f - 1.) / (f - cr)
                } else {
                    eff / (f64::from(S) - eff * (f64::from(S) - 1.))
                };
                ntu_1(eff_1, cr)
            })
        }
    }
}

/// Errors returned when constructing a [`ShellAndTube`] arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAndTubeConfigError {
    /// No shell passes were configured.
    ZeroShellPasses,
    /// The requested shell pass count is too large to validate.
    ShellPassOverflow,
    /// Tube passes are fewer than twice the shell passes.
    InsufficientTubePasses,
    /// Tube passes are not an even multiple of shell passes.
    TubePassesNotMultiple,
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use twine_core::constraint::ConstraintResult;
    use uom::si::{ratio::ratio, thermal_conductance::watt_per_kelvin};

    use crate::thermal::hx::{CapacitanceRate, Ntu};

    use super::*;

    fn roundtrip_for<const N: u16, const T: u16>() -> ConstraintResult<()> {
        let arrangement =
            ShellAndTube::<N, T>::new().expect("shell-and-tube configuration should be valid");

        let ntus = [0.1, 0.5, 1., 5.];
        let capacitance_rates = [[1., 1.], [1., 2.], [2., 1.], [1., 4.]];

        for ntu in ntus {
            for pair in capacitance_rates {
                let rates = [
                    CapacitanceRate::new::<watt_per_kelvin>(pair[0])?,
                    CapacitanceRate::new::<watt_per_kelvin>(pair[1])?,
                ];

                let eff = arrangement.effectiveness(Ntu::new(ntu)?, rates);
                let back = arrangement.ntu(eff, rates);

                assert_relative_eq!(back.get::<ratio>(), ntu, max_relative = 1e-12);
            }
        }

        Ok(())
    }

    #[test]
    fn validation_outcomes() {
        const MAX: u16 = u16::MAX;

        assert_eq!(
            ShellAndTube::<0, 2>::new(),
            Err(ShellAndTubeConfigError::ZeroShellPasses)
        );

        assert_eq!(
            ShellAndTube::<MAX, 2>::new(),
            Err(ShellAndTubeConfigError::ShellPassOverflow)
        );

        assert_eq!(
            ShellAndTube::<3, 4>::new(),
            Err(ShellAndTubeConfigError::InsufficientTubePasses)
        );

        assert_eq!(
            ShellAndTube::<3, 8>::new(),
            Err(ShellAndTubeConfigError::TubePassesNotMultiple)
        );

        assert!(ShellAndTube::<1, 2>::new().is_ok());
    }

    #[test]
    fn roundtrip() -> ConstraintResult<()> {
        roundtrip_for::<1, 2>()?;
        roundtrip_for::<1, 4>()?;
        roundtrip_for::<2, 4>()?;
        roundtrip_for::<3, 12>()?;

        Ok(())
    }
}
