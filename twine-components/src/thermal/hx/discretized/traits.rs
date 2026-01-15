//! Internal traits for discretized heat exchanger solving.

use twine_thermo::{
    capability::{HasEnthalpy, HasPressure, StateFrom, ThermoModel},
    units::SpecificEnthalpy,
};
use uom::si::f64::{Pressure, ThermodynamicTemperature};

use crate::thermal::hx::{
    arrangement::{CounterFlow, ParallelFlow},
    effectiveness_ntu::NtuRelation,
};

/// Arrangement contract for discretized solvers.
///
/// This trait extends [`NtuRelation`] with the bottom stream flow direction,
/// which is required to discretize the heat exchanger into nodes.
///
/// The "bottom" stream refers to the physical position in the discretized
/// model, not necessarily the hot or cold stream.
/// The top stream always flows left to right (node 0 to node N-1).
/// The bottom stream's direction depends on the arrangement.
#[doc(hidden)]
pub trait DiscretizedArrangement: NtuRelation {
    /// True if the bottom stream flows left-to-right (node 0 to N-1).
    ///
    /// False if it flows right-to-left (node N-1 to 0).
    const BOTTOM_FLOWS_LEFT_TO_RIGHT: bool;

    /// Selects a value based on bottom stream flow direction.
    ///
    /// Returns `forward` if the bottom stream flows left-to-right, `reverse` otherwise.
    #[inline]
    fn bottom_select<T>(forward: T, reverse: T) -> T {
        if Self::BOTTOM_FLOWS_LEFT_TO_RIGHT {
            forward
        } else {
            reverse
        }
    }
}

impl DiscretizedArrangement for CounterFlow {
    const BOTTOM_FLOWS_LEFT_TO_RIGHT: bool = false;
}

impl DiscretizedArrangement for ParallelFlow {
    const BOTTOM_FLOWS_LEFT_TO_RIGHT: bool = true;
}

/// Required thermo model bounds for discretized heat exchangers.
#[doc(hidden)]
pub trait DiscretizedHxThermoModel<Fluid>:
    ThermoModel<Fluid = Fluid>
    + HasPressure
    + HasEnthalpy
    + StateFrom<(Fluid, ThermodynamicTemperature, Pressure)>
    + StateFrom<(Fluid, Pressure, SpecificEnthalpy)>
{
}

impl<Fluid, T> DiscretizedHxThermoModel<Fluid> for T where
    T: ThermoModel<Fluid = Fluid>
        + HasPressure
        + HasEnthalpy
        + StateFrom<(Fluid, ThermodynamicTemperature, Pressure)>
        + StateFrom<(Fluid, Pressure, SpecificEnthalpy)>
{
}
