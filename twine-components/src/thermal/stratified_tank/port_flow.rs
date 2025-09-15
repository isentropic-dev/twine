use twine_core::constraint::{Constrained, ConstraintError, NonNegative};
use uom::si::f64::{ThermodynamicTemperature, VolumeRate};

/// Inlet conditions for a port pair.
///
/// See [`StratifiedTank`] for how port pairs are placed and distributed across nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortFlow {
    /// Common volumetric flow for the port pair (inlet and outlet).
    pub rate: Constrained<VolumeRate, NonNegative>,

    /// Inlet fluid temperature.
    ///
    /// The outlet temperature is determined by the node(s) assocated with the outflow.
    pub inlet_temperature: ThermodynamicTemperature,
}

impl PortFlow {
    /// Creates a `PortFlow` from an unconstrained rate.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `rate` is negative.
    pub fn new(
        rate: VolumeRate,
        inlet_temperature: ThermodynamicTemperature,
    ) -> Result<Self, ConstraintError> {
        let rate = Constrained::new(rate)?;
        Ok(Self::from_constrained(rate, inlet_temperature))
    }

    /// Creates a `PortFlow` from a constrained rate.
    #[must_use]
    pub fn from_constrained(
        rate: Constrained<VolumeRate, NonNegative>,
        inlet_temperature: ThermodynamicTemperature,
    ) -> Self {
        Self {
            rate,
            inlet_temperature,
        }
    }

    /// Returns the volumetric flow shared by the inlet and outlet.
    #[must_use]
    pub fn rate(&self) -> VolumeRate {
        self.rate.into_inner()
    }

    /// Extracts the flow rate from this port flow, consuming self.
    #[must_use]
    pub fn into_rate(self) -> VolumeRate {
        self.rate.into_inner()
    }
}
