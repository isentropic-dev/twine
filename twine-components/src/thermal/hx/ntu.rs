use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, NonNegative};
use uom::si::{
    f64::{Ratio, ThermalConductance},
    ratio::ratio,
};

use crate::thermal::hx::{
    capacitance_rate::CapacitanceRate, capacity_ratio::CapacityRatio, effectiveness::Effectiveness,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ntu(Constrained<Ratio, NonNegative>);

impl Ntu {
    pub fn new(value: f64) -> ConstraintResult<Self> {
        let quantity = Ratio::new::<ratio>(value);
        Self::from_quantity(quantity)
    }

    pub fn from_quantity(quantity: Ratio) -> ConstraintResult<Self> {
        Ok(Self(NonNegative::new(quantity)?))
    }

    pub fn from_conductance_and_capacitance_rates(
        ua: ThermalConductance,
        capacitance_rates: [CapacitanceRate; 2],
    ) -> ConstraintResult<Self> {
        Self::from_quantity(ua / capacitance_rates[0].min(*capacitance_rates[1]))
    }

    fn infinite_capacitance_rate(effectiveness: Effectiveness) -> Self {
        let eff = effectiveness.get::<ratio>();
        Self::new(-(1. - eff).ln()).expect("effectiveness should always yield valid ntu")
    }

    pub(super) fn counter_flow(
        effectiveness: Effectiveness,
        capacity_ratio: CapacityRatio,
    ) -> Self {
        let cr = capacity_ratio.get::<ratio>();

        if cr == 0. {
            return Self::infinite_capacitance_rate(effectiveness);
        }

        Self::new({
            let eff = effectiveness.get::<ratio>();
            if cr < 1. {
                (((1. - eff * cr) / (1. - eff)).ln()) / (1. - cr)
            } else {
                // cr == 1
                eff / (1. - eff)
            }
        })
        .expect("effectiveness should always yield valid ntu")
    }
}

impl Deref for Ntu {
    type Target = Ratio;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
