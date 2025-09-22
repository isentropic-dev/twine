use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, UnitInterval};
use uom::si::{f64::Ratio, ratio::ratio};

use crate::thermal::hx::capacitance_rate::CapacitanceRate;

#[derive(Debug, Clone, Copy)]
pub struct CapacityRatio(Constrained<Ratio, UnitInterval>);

impl CapacityRatio {
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(UnitInterval::new(quantity)?))
    }

    pub fn from_capacitance_rates(
        capacitance_rates: [CapacitanceRate; 2],
    ) -> ConstraintResult<Self> {
        let [first, second] = capacitance_rates;

        Self::from_quantity(first.min(*second) / first.max(*second))
    }
}

impl Deref for CapacityRatio {
    type Target = Ratio;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
