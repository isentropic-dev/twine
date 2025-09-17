#![allow(dead_code)]

use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintError, NonNegative, UnitInterval};
use uom::si::{f64::ThermalConductance, thermal_conductance::watt_per_kelvin};

/// A fluid capacitance rate.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct CapacitanceRate(Constrained<ThermalConductance, NonNegative>);

impl CapacitanceRate {
    /// Constructs a `CapacitanceRate`.
    ///
    /// # Errors
    ///
    /// Fails if the value is negative.
    fn new(value: ThermalConductance) -> Result<Self, ConstraintError> {
        Ok(Self(NonNegative::new(value)?))
    }
}

impl TryFrom<ThermalConductance> for CapacitanceRate {
    type Error = ConstraintError;

    fn try_from(value: ThermalConductance) -> Result<Self, Self::Error> {
        CapacitanceRate::new(value)
    }
}

impl Deref for CapacitanceRate {
    type Target = ThermalConductance;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// The capacity ratio of two fluids in a heat exchanger.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct CapacityRatio(Constrained<f64, UnitInterval>);

impl CapacityRatio {
    /// Constructs a `CapacityRatio`.
    ///
    /// # Errors
    ///
    /// Fails if the value is not in the unit interval.
    fn new(value: f64) -> Result<Self, ConstraintError> {
        Ok(Self(UnitInterval::new(value)?))
    }

    /// Construct a `CapacityRatio` from the `CapacitanceRate`s of two fluids in
    /// a heat exchanger.
    fn from_capacitance_rates(
        c_dot_cold: CapacitanceRate,
        c_dot_hot: CapacitanceRate,
    ) -> Result<Self, ConstraintError> {
        let (c_dot_min, c_dot_max) = if c_dot_cold <= c_dot_hot {
            (c_dot_cold, c_dot_hot)
        } else {
            (c_dot_hot, c_dot_cold)
        };

        Self::new(c_dot_min.get::<watt_per_kelvin>() / c_dot_max.get::<watt_per_kelvin>())
    }
}

impl TryFrom<f64> for CapacityRatio {
    type Error = ConstraintError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_capacitance_rates() -> Result<(), ConstraintError> {
        let cases = &[(0., 100., 0.), (100., 100., 1.), (25., 100., 0.25)];

        for &(cold, hot, expected) in cases {
            let cr = CapacityRatio::from_capacitance_rates(
                ThermalConductance::new::<watt_per_kelvin>(cold).try_into()?,
                ThermalConductance::new::<watt_per_kelvin>(hot).try_into()?,
            )?;

            assert_eq!(cr, expected.try_into()?);
        }

        Ok(())
    }

    #[test]
    fn from_capacitance_rates_both_zero() -> Result<(), ConstraintError> {
        let result = CapacityRatio::from_capacitance_rates(
            ThermalConductance::new::<watt_per_kelvin>(0.).try_into()?,
            ThermalConductance::new::<watt_per_kelvin>(0.).try_into()?,
        );

        assert!(matches!(result, Err(ConstraintError::NotANumber)));

        Ok(())
    }
}
