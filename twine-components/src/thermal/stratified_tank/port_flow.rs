use twine_core::constraint::{Constrained, ConstraintError, NonNegative};
use uom::si::f64::{ThermodynamicTemperature, VolumeRate};

/// Inlet conditions for a port pair.
///
/// See [`StratifiedTank`] for how port pairs are placed and distributed across layers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortFlow {
    /// Common volumetric flow for the port pair (inlet and outlet).
    pub rate: Constrained<VolumeRate, NonNegative>,

    /// Inlet fluid temperature.
    ///
    /// The outlet temperature is determined by the layer assocated with the outflow.
    pub temperature: ThermodynamicTemperature,
}

impl PortFlow {
    /// Creates a `PortFlow` from an unconstrained rate.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `rate` is negative.
    pub fn new(
        rate: VolumeRate,
        temperature: ThermodynamicTemperature,
    ) -> Result<Self, ConstraintError> {
        let rate = Constrained::new(rate)?;
        Ok(Self::from_constrained(rate, temperature))
    }

    /// Creates a `PortFlow` from a constrained rate.
    #[must_use]
    pub fn from_constrained(
        rate: Constrained<VolumeRate, NonNegative>,
        temperature: ThermodynamicTemperature,
    ) -> Self {
        Self { rate, temperature }
    }

    /// Returns the volumetric flow shared by the inlet and outlet.
    #[must_use]
    pub fn rate(&self) -> VolumeRate {
        self.rate.into_inner()
    }
}
