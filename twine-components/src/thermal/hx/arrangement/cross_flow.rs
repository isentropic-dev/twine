//! Cross-flow effectiveness-NTU relationships.

use crate::thermal::hx::effectiveness_ntu::{
    EffectivenessRelation, NtuRelation, effectiveness_via, ntu_via,
};

/// Cross-flow heat exchanger arrangement.
#[derive(Debug, Clone, Copy)]
pub struct CrossFlow<T: MixState, U: MixState>(T, U);

/// Marker type for a cross-flow stream that is mixed across the flow channel.
pub struct Mixed;
/// Marker type for a cross-flow stream that remains unmixed across the flow channel.
pub struct Unmixed;

pub trait MixState {}
impl MixState for Mixed {}
impl MixState for Unmixed {}

impl EffectivenessRelation for CrossFlow<Unmixed, Unmixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        effectiveness_via(ntu, capacitance_rates, |ntu, cr| {
            1. - ((ntu.powf(0.22) / cr) * ((-cr * ntu.powf(0.78)).exp() - 1.)).exp()
        })
    }
}

impl EffectivenessRelation for CrossFlow<Mixed, Mixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        effectiveness_via(ntu, capacitance_rates, |ntu, cr| {
            1. / (1. / (1. - (-ntu).exp()) + cr / (1. - (-cr * ntu).exp()) - 1. / ntu)
        })
    }
}

impl EffectivenessRelation for CrossFlow<Mixed, Unmixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        if capacitance_rates[0] >= capacitance_rates[1] {
            effectiveness_via(ntu, capacitance_rates, |ntu, cr| {
                (1. - (cr * ((-ntu).exp() - 1.)).exp()) / cr
            })
        } else {
            effectiveness_via(ntu, capacitance_rates, |ntu, cr| {
                1. - (-((1. - (-cr * ntu).exp()) / cr)).exp()
            })
        }
    }
}

impl EffectivenessRelation for CrossFlow<Unmixed, Mixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        CrossFlow(Mixed, Unmixed).effectiveness(ntu, [capacitance_rates[1], capacitance_rates[0]])
    }
}

impl NtuRelation for CrossFlow<Mixed, Unmixed> {
    fn ntu(
        &self,
        effectiveness: crate::thermal::hx::Effectiveness,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Ntu {
        if capacitance_rates[0] >= capacitance_rates[1] {
            ntu_via(effectiveness, capacitance_rates, |eff, cr| {
                -(1. + (1. - eff * cr).ln() / cr).ln()
            })
        } else {
            ntu_via(effectiveness, capacitance_rates, |eff, cr| {
                -(cr * (1. - eff).ln() + 1.).ln() / cr
            })
        }
    }
}

impl NtuRelation for CrossFlow<Unmixed, Mixed> {
    fn ntu(
        &self,
        effectiveness: crate::thermal::hx::Effectiveness,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Ntu {
        CrossFlow(Mixed, Unmixed).ntu(effectiveness, [capacitance_rates[1], capacitance_rates[0]])
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use twine_core::constraint::ConstraintResult;
    use uom::si::{ratio::ratio, thermal_conductance::watt_per_kelvin};

    use crate::thermal::hx::{CapacitanceRate, Ntu};

    use super::*;

    #[test]
    fn roundtrip_mixed_unmixed() -> ConstraintResult<()> {
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
            // flip mixed/unmixed
            // c_r == 0.5
            [2., 1.],
            // c_r == 0.25
            [4., 1.],
            // c_r == 0
            [f64::INFINITY, 1.],
        ];

        for ntu in ntus {
            for pair in capacitance_rates {
                let rates = [
                    CapacitanceRate::new::<watt_per_kelvin>(pair[0])?,
                    CapacitanceRate::new::<watt_per_kelvin>(pair[1])?,
                ];

                let mixed_unmixed = CrossFlow(Mixed, Unmixed);
                let eff = mixed_unmixed.effectiveness(Ntu::new(ntu)?, rates);
                let back = mixed_unmixed.ntu(eff, rates);

                assert_relative_eq!(back.get::<ratio>(), ntu, max_relative = 1e-12);
            }
        }

        Ok(())
    }
}
