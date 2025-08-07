use twine_core::constraint::{Constrained, ConstraintError, NonNegative};
use twine_thermo::model::incompressible::IncompressibleFluid;
use uom::si::f64::{MassRate, ThermodynamicTemperature, VolumeRate};

/// Fluid flow into the tank through a configured inlet port.
///
/// Represents fluid entering a specific layer in the tank at a known
/// temperature and volume flow rate.
/// A corresponding outflow occurs elsewhere in the tank.
///
/// The volume flow rate is guaranteed to be non-negative and is validated at construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortFlow {
    rate: VolumeRate,
    temperature: ThermodynamicTemperature,
}

impl PortFlow {
    /// Creates a new [`PortFlow`] with the given volume rate and temperature.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `rate` is negative or not finite.
    pub fn new(
        rate: VolumeRate,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self, ConstraintError> {
        Constrained::<VolumeRate, NonNegative>::new(rate)?;
        Ok(Self { rate, temperature })
    }

    /// Creates a [`PortFlow`] from a pre-validated constrained volume rate.
    ///
    /// Useful when working with existing [`Constrained`] values.
    #[must_use]
    pub fn from_constrained(
        rate: Constrained<VolumeRate, NonNegative>,
        temperature: ThermodynamicTemperature,
    ) -> Self {
        Self {
            rate: rate.into_inner(),
            temperature,
        }
    }

    /// Returns the volume flow rate.
    ///
    /// Guaranteed to be non-negative.
    #[must_use]
    pub fn rate(&self) -> VolumeRate {
        self.rate
    }

    /// Returns the temperature of the incoming fluid.
    #[must_use]
    pub fn temperature(&self) -> ThermodynamicTemperature {
        self.temperature
    }

    /// Returns the mass flow rate using the given fluid's density.
    ///
    /// Guaranteed to be non-negative.
    #[must_use]
    pub fn mass_rate<F: IncompressibleFluid>(&self, fluid: &F) -> MassRate {
        self.rate * fluid.reference_density()
    }
}
