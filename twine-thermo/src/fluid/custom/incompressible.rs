use uom::si::f64::{MassDensity, SpecificHeatCapacity, ThermodynamicTemperature};

use crate::model::incompressible::IncompressibleFluid;

/// User-defined incompressible fluid.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct IncompressibleCustom {
    /// Optional name for identification.
    pub name: Option<String>,

    /// Specific heat capacity.
    pub specific_heat: SpecificHeatCapacity,

    /// Reference temperature for enthalpy and entropy calculations.
    pub reference_temperature: ThermodynamicTemperature,

    /// Reference density.
    pub reference_density: MassDensity,
}

impl IncompressibleFluid for IncompressibleCustom {
    fn specific_heat(&self) -> SpecificHeatCapacity {
        self.specific_heat
    }

    fn reference_temperature(&self) -> ThermodynamicTemperature {
        self.reference_temperature
    }

    fn reference_density(&self) -> MassDensity {
        self.reference_density
    }
}
