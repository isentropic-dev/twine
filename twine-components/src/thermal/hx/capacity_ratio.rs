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

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use uom::si::thermal_conductance::watt_per_kelvin;

    use super::*;

    #[test]
    fn from_capacitance_rates() -> ConstraintResult<()> {
        let capacitance_rates = [
            CapacitanceRate::new::<watt_per_kelvin>(10.)?,
            CapacitanceRate::new::<watt_per_kelvin>(20.)?,
        ];

        let capacity_ratio = CapacityRatio::from_capacitance_rates(capacitance_rates)?;

        assert_relative_eq!(capacity_ratio.get::<ratio>(), 0.5);
        Ok(())
    }
}
