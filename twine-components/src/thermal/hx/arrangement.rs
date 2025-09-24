//! Flow arrangements supported by the heat exchanger utilities.

mod counterflow;

pub use counterflow::CounterFlow;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thermal::hx::{CapacityRatio, EffectivenessNtu, effectiveness_ntu::Ntu};

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
