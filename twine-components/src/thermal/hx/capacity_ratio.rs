use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, UnitInterval};
use uom::si::{f64::Ratio, ratio::ratio};

use crate::thermal::hx::capacitance_rate::CapacitanceRate;

/// Capacity ratio (`C_min` / `C_max`) for a heat exchanger.
///
/// The ratio quantifies how evenly the stream capacitance rates are matched and
/// must fall in the closed interval [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct CapacityRatio(Constrained<Ratio, UnitInterval>);

impl CapacityRatio {
    /// Create a [`CapacityRatio`] from a scalar value.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the value lies outside the interval [0, 1].
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    /// Create a [`CapacityRatio`] from a quantity with ratio units.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the quantity lies outside the interval [0, 1].
    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(UnitInterval::new(quantity)?))
    }

    /// Create a [`CapacityRatio`] from the [capacitance rates](CapacitanceRate)
    /// of the two streams.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the ratio of the supplied capacitance rates falls
    /// outside the interval [0, 1].
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
