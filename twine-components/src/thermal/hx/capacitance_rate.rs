use std::ops::{Deref, Div};

use twine_core::constraint::{Constrained, ConstraintResult, StrictlyPositive};
use uom::si::f64::{
    MassRate, Power, SpecificHeatCapacity, TemperatureInterval, ThermalConductance,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CapacitanceRate(Constrained<ThermalConductance, StrictlyPositive>);

impl CapacitanceRate {
    pub fn new<U>(value: f64) -> ConstraintResult<Self>
    where
        U: uom::si::thermal_conductance::Unit + uom::Conversion<f64, T = f64>,
    {
        let quantity = ThermalConductance::new::<U>(value);
        Self::from_quantity(quantity)
    }

    pub fn from_quantity(quantity: ThermalConductance) -> ConstraintResult<Self> {
        Ok(Self(StrictlyPositive::new(quantity)?))
    }

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

impl Div<CapacitanceRate> for Power {
    type Output = TemperatureInterval;

    fn div(self, rhs: CapacitanceRate) -> Self::Output {
        self / rhs.0.into_inner()
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
