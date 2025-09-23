use std::ops::Deref;

use twine_core::constraint::{Constrained, ConstraintResult, StrictlyPositive};
use uom::si::f64::{MassRate, SpecificHeatCapacity, ThermalConductance};

/// The capacitance rate of a working fluid in a heat exchanger.
///
/// The capacitance rate must be > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CapacitanceRate(Constrained<ThermalConductance, StrictlyPositive>);

impl CapacitanceRate {
    /// Create a [`CapacitanceRate`] from a value.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value is <= 0.
    pub fn new<U>(value: f64) -> ConstraintResult<Self>
    where
        U: uom::si::thermal_conductance::Unit + uom::Conversion<f64, T = f64>,
    {
        let quantity = ThermalConductance::new::<U>(value);
        Self::from_quantity(quantity)
    }

    /// Create a [`CapacitanceRate`] from a uom quantity.
    ///
    /// # Errors
    ///
    /// This function will return an error if the quantity is <= 0.
    pub fn from_quantity(quantity: ThermalConductance) -> ConstraintResult<Self> {
        Ok(Self(StrictlyPositive::new(quantity)?))
    }

    /// Create a [`CapacitanceRate`] from a mass rate and specific heat
    /// capacity.
    ///
    /// # Errors
    ///
    /// This function will return an error if either the mass rate or specific
    /// heat is <= 0.
    pub fn from_mass_rate_and_specific_heat(
        mass_rate: MassRate,
        specific_heat: SpecificHeatCapacity,
    ) -> ConstraintResult<Self> {
        CapacitanceRate::from_quantity(mass_rate * specific_heat)
    }
}

impl Deref for CapacitanceRate {
    type Target = ThermalConductance;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use uom::si::{
        mass_rate::kilogram_per_second, specific_heat_capacity::joule_per_kilogram_kelvin,
        thermal_conductance::watt_per_kelvin,
    };

    use super::*;

    #[test]
    fn from_mass_rate_and_specific_heat() -> ConstraintResult<()> {
        let mass_rate = MassRate::new::<kilogram_per_second>(10.);
        let specific_heat = SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(4000.);

        let capacitance_rate =
            CapacitanceRate::from_mass_rate_and_specific_heat(mass_rate, specific_heat)?;

        assert_relative_eq!(capacitance_rate.get::<watt_per_kelvin>(), 40000.);
        Ok(())
    }
}
