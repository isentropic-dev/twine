use std::marker::PhantomData;

use uom::si::ratio::ratio;

use crate::thermal::hx::{
    CapacitanceRate, Effectiveness, Ntu,
    effectiveness_ntu::{EffectivenessRelation, NtuRelation, effectiveness_via, ntu_via},
};

/// Shell-and-tube heat exchanger arrangement constrained to supported pass counts.
#[derive(Debug, Clone, Copy)]
pub struct ShellAndTube<const S: i32, const T: i32> {
    _marker: PhantomData<()>,
}

impl<const N: i32, const T: i32> ShellAndTube<N, T> {
    const fn assert_valid() {
        assert!(
            N > 0,
            "shell-and-tube exchangers require at least one shell pass"
        );
        assert!(
            N <= i32::MAX / 2,
            "shell pass count is too large to evaluate compile-time constraints",
        );
        assert!(
            T >= 2 * N,
            "tube passes must be at least twice the shell passes"
        );
        assert!(
            T % (2 * N) == 0,
            "tube passes must be an even multiple of the shell passes",
        );
    }

    /// Construct a validated shell-and-tube arrangement configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self::assert_valid();
        Self {
            _marker: PhantomData,
        }
    }
}

impl<const N: i32, const T: i32> Default for ShellAndTube<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: i32, const T: i32> EffectivenessRelation for ShellAndTube<N, T> {
    fn effectiveness(&self, ntu: Ntu, capacitance_rates: [CapacitanceRate; 2]) -> Effectiveness {
        Self::assert_valid();
        let eff_1 = effectiveness_via(ntu, capacitance_rates, |ntu_1, cr| {
            2. * 1.
                / (1.
                    + cr
                    + (1. + cr.powi(2)).sqrt() * (1. + (-ntu_1 * (1. + cr.powi(2)).sqrt()).exp())
                        / (1. - (-ntu_1 * (1. + cr.powi(2)).sqrt()).exp()))
        });

        if N == 1 {
            eff_1
        } else {
            effectiveness_via(ntu, capacitance_rates, |_, cr| {
                let eff_1 = eff_1.get::<ratio>();
                (((1. - eff_1 * cr) / (1. - eff_1)).powi(N) - 1.)
                    / (((1. - eff_1 * cr) / (1. - eff_1)).powi(N) - cr)
            })
        }
    }
}

impl<const N: i32, const T: i32> NtuRelation for ShellAndTube<N, T> {
    fn ntu(&self, effectiveness: Effectiveness, capacitance_rates: [CapacitanceRate; 2]) -> Ntu {
        Self::assert_valid();

        let ntu_1: fn(f64, f64) -> f64 = |eff_1, cr| {
            let e = (2. - eff_1 * (1. + cr)) / (eff_1 * (1. + cr.powi(2)).sqrt());
            ((e + 1.) / (e - 1.)).ln() / (1. + cr.powi(2)).sqrt()
        };

        if N == 1 {
            ntu_via(effectiveness, capacitance_rates, ntu_1)
        } else {
            ntu_via(effectiveness, capacitance_rates, |eff, cr| {
                let f = ((eff * cr - 1.) / (eff - 1.)).powi(1 / N);
                let eff_1 = (f - 1.) / (f - cr);
                ntu_1(eff_1, cr)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const _: ShellAndTube<1, 2> = ShellAndTube::new();

    #[test]
    fn constructs_valid_configuration() {
        let _ = ShellAndTube::<2, 8>::new();
    }
}
