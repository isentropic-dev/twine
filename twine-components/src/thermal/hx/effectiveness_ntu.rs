use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, NonNegative, UnitInterval};
use uom::si::{
    f64::{Ratio, ThermalConductance},
    ratio::ratio,
};

use crate::thermal::hx::{CapacitanceRate, CapacityRatio};

pub trait EffectivenessNtu {
    /// Calculate the effectiveness for an arrangement given the [NTU](Ntu) and
    ///[capacity ratio](CapacityRatio).
    fn effectiveness(&self, ntu: Ntu, capacity_ratio: CapacityRatio) -> Effectiveness;

    /// Calculate the [NTU](Ntu) for an arrangement given the
    /// [effectiveness](Effectiveness) and [capacity ratio](CapacityRatio).
    fn ntu(&self, effectiveness: Effectiveness, capacity_ratio: CapacityRatio) -> Ntu;
}

/// The effectiveness of a heat exchanger.
///
/// The effectiveness is the ratio of the actual amount of heat transferred to
/// the maximum possible amount of heat transferred in the heat exchanger.
///
/// The effectiveness must be in the interval [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct Effectiveness(Constrained<Ratio, UnitInterval>);

impl Effectiveness {
    /// Create an [`Effectiveness`] from a value.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value is not in the interval
    /// [0, 1].
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    /// Create an [`Effectiveness`] from a uom quantity.
    ///
    /// # Errors
    ///
    /// This function will return an error if the quantity is not in the
    /// interval [0, 1].
    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(UnitInterval::new(quantity)?))
    }

    pub(super) fn infinite_capacitance_rate(ntu: Ntu) -> Self {
        let ntu = ntu.get::<ratio>();
        Self::new(1. - (-ntu).exp()).expect("ntu should always yield valid effectiveness")
    }
}

impl Deref for Effectiveness {
    type Target = Ratio;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// The number of transfer units for a heat exchanger.
///
/// The number of transfer units represents the dimensionless size of a heat
/// exchanger.
///
/// The number of transfer units must be >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ntu(Constrained<Ratio, NonNegative>);

impl Ntu {
    /// Create an [`Ntu`] from a value.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value is < 0.
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    /// Create an [`Ntu`] from a uom quantity.
    ///
    /// # Errors
    ///
    /// This function will return an error if the quantity is < 0.
    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(NonNegative::new(quantity)?))
    }

    /// Create an [`Ntu`] from a heat exchanger conductance and
    /// [capacitance rates](CapacitanceRate).
    ///
    /// The [capacitance rates](CapacitanceRate) of both streams are required so
    /// that the minimum of the two can be used in the calculation.
    ///
    /// # Errors
    ///
    /// This function will return an error if either the conductance or
    /// [capacitance rates](CapacitanceRate) are < 0.
    pub fn from_conductance_and_capacitance_rates(
        ua: ThermalConductance,
        capacitance_rates: [CapacitanceRate; 2],
    ) -> ConstraintResult<Self> {
        Self::from_quantity(ua / capacitance_rates[0].min(*capacitance_rates[1]))
    }

    pub(super) fn infinite_capacitance_rate(effectiveness: Effectiveness) -> Self {
        let eff = effectiveness.get::<ratio>();
        Self::new(-(1. - eff).ln()).expect("effectiveness should always yield valid ntu")
    }
}

impl Deref for Ntu {
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
    fn ntu_from_conductance_and_capacitance_rates() -> ConstraintResult<()> {
        let ua = ThermalConductance::new::<watt_per_kelvin>(10.);
        let capacitance_rates = [
            CapacitanceRate::new::<watt_per_kelvin>(10.)?,
            CapacitanceRate::new::<watt_per_kelvin>(20.)?,
        ];

        let ntu = Ntu::from_conductance_and_capacitance_rates(ua, capacitance_rates)?;

        assert_relative_eq!(ntu.get::<ratio>(), 1.);
        Ok(())
    }
}
