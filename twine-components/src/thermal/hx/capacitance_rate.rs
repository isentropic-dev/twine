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
