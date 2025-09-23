use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, UnitInterval};
use uom::si::{f64::Ratio, ratio::ratio};

use crate::thermal::hx::capacitance_rate::CapacitanceRate;

/// The capacity ratio of a heat exchanger.
///
/// The capacity ratio is a dimensionless number reflecting how well-balanced a
/// heat exchanger is. If a heat exchanger is perfectly balanced (capacity ratio
/// = 1), then both fluids will experience the same temperature change. If a
/// heat exchanger is unbalanced, one fluid will experience a larger temperature
/// change than the other.
///
/// The capacity ratio must be in the interval [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct CapacityRatio(Constrained<Ratio, UnitInterval>);

impl CapacityRatio {
    /// Create a [`CapacityRatio`] from a value.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value is not in the interval
    /// [0, 1].
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    /// Create a [`CapacityRatio`] from a uom quantity.
    ///
    /// # Errors
    ///
    /// This function will return an error if the quantity is not in the
    /// interval [0, 1].
    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(UnitInterval::new(quantity)?))
    }

    /// Create a [`CapacityRatio`] from the [capacitance rates](CapacitanceRate)
    /// of the fluids.
    ///
    /// # Errors
    ///
    /// This function will return an error if either capacitance rate is not in
    /// the interval [0, 1].
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
