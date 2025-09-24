use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, UnitInterval};
use uom::si::{f64::Ratio, ratio::ratio};

use crate::thermal::hx::ntu::Ntu;

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
