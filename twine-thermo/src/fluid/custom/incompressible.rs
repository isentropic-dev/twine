use twine_core::TimeIntegrable;
use uom::si::f64::{MassDensity, SpecificHeatCapacity, ThermalConductivity, Time};

use crate::model::incompressible::IncompressibleFluid;

/// User-defined incompressible fluid.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct IncompressibleCustom {
    /// Optional name for identification.
    pub name: Option<String>,
    pub density: MassDensity,
    pub specific_heat: SpecificHeatCapacity,
    pub thermal_conductivity: ThermalConductivity,
}

impl IncompressibleFluid for IncompressibleCustom {
    fn density(&self) -> MassDensity {
        self.density
    }

    fn specific_heat(&self) -> SpecificHeatCapacity {
        self.specific_heat
    }

    fn thermal_conductivity(&self) -> ThermalConductivity {
        self.thermal_conductivity
    }
}

impl TimeIntegrable for IncompressibleCustom {
    type Derivative = ();

    fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
        self
    }
}
