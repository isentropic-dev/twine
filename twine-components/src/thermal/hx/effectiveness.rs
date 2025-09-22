use std::ops::{Deref, Mul};

use twine_core::constraint::{Constrained, ConstraintResult, UnitInterval};
use uom::si::{
    f64::{Power, Ratio},
    ratio::ratio,
};

use crate::thermal::hx::{capacity_ratio::CapacityRatio, ntu::Ntu};

#[derive(Debug, Clone, Copy)]
pub struct Effectiveness(Constrained<Ratio, UnitInterval>);

impl Effectiveness {
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(UnitInterval::new(quantity)?))
    }

    fn infinite_capacitance_rate(ntu: Ntu) -> Self {
        let ntu = ntu.get::<ratio>();
        Self::new(1. - (-ntu).exp()).expect("ntu should always yield valid effectiveness")
    }

    pub(super) fn counter_flow(ntu: Ntu, capacity_ratio: CapacityRatio) -> Self {
        let cr = capacity_ratio.get::<ratio>();

        if cr == 0. {
            return Self::infinite_capacitance_rate(ntu);
        }

        Self::new({
            let ntu = ntu.get::<ratio>();
            if cr < 1. {
                (1. - (-ntu * (1. - cr)).exp()) / (1. - cr * (-ntu * (1. - cr)).exp())
            } else {
                ntu / (1. + ntu)
            }
        })
        .expect("ntu should always yield valid effectiveness")
    }
}

impl Deref for Effectiveness {
    type Target = Ratio;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl Mul<Power> for Effectiveness {
    type Output = Power;

    fn mul(self, rhs: Power) -> Self::Output {
        self.0.into_inner() * rhs
    }
}
