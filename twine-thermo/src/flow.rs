use twine_core::constraint::{Constrained, NonNegative};
use uom::si::f64::MassRate;

use crate::State;

/// A flow of fluid at a given thermodynamic state.
///
/// A `Flow<Fluid>` combines a mass flow rate with a [`State<Fluid>`] to
/// represent directional fluid flow in thermodynamic systems.
///
/// The mass flow rate is constrained to be non-negative, ensuring that each
/// flow has a single, well-defined direction.
#[derive(Debug, Clone, PartialEq)]
pub struct Flow<Fluid> {
    pub mass_rate: Constrained<MassRate, NonNegative>,
    pub state: State<Fluid>,
}

impl<Fluid> Flow<Fluid> {
    /// Creates a new `Flow` from a mass flow rate and thermodynamic state.
    pub fn new(mass_rate: Constrained<MassRate, NonNegative>, state: State<Fluid>) -> Self {
        Self { mass_rate, state }
    }
}
