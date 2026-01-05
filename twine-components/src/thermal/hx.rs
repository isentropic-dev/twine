#![warn(missing_docs)]

//! Tools for analyzing heat exchangers.
//!
//! These utilities provide standard effectiveness-NTU relationships, helpers
//! for sizing exchangers, and functional APIs for working directly with
//! thermodynamic primitives.

pub mod arrangement;
mod capacitance_rate;
mod capacity_ratio;
mod effectiveness_ntu;
pub mod functional;
mod stream;

pub use capacitance_rate::CapacitanceRate;
pub use capacity_ratio::CapacityRatio;
pub use effectiveness_ntu::{Effectiveness, Ntu};
pub use stream::StreamInlet;
use twine_core::constraint::ConstraintResult;
use uom::si::f64::ThermalConductance;

use crate::thermal::hx::{
    effectiveness_ntu::EffectivenessRelation, functional::KnownConductanceResult,
};

/// High-level entry point for solving heat exchanger scenarios with a chosen
/// arrangement.
///
/// The wrapped arrangement must implement [`EffectivenessNtu`], providing the
/// effectiveness/NTU relationships consumed by helper methods. Calculations
/// assume both fluids maintain a constant specific heat as they traverse the
/// exchanger.
///
/// # Example
///
/// ```rust
/// # use twine_core::constraint::ConstraintResult;
/// use uom::si::{
///     f64::{ThermalConductance, ThermodynamicTemperature},
///     thermal_conductance::kilowatt_per_kelvin,
///     thermodynamic_temperature::degree_celsius,
/// };
/// use twine_components::thermal::hx::{arrangement, CapacitanceRate, Hx, StreamInlet};
///
/// # fn main() -> ConstraintResult<()> {
/// let hx = Hx::new(arrangement::CounterFlow);
///
/// let _ = hx.known_conductance_and_inlets(
///     ThermalConductance::new::<kilowatt_per_kelvin>(3. * 4.0_f64.ln()),
///     [
///         StreamInlet::new(
///             CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
///             ThermodynamicTemperature::new::<degree_celsius>(50.),
///         ),
///         StreamInlet::new(
///             CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
///             ThermodynamicTemperature::new::<degree_celsius>(80.),
///         ),
///     ],
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct Hx<T>(T);

impl<T> Hx<T> {
    /// Create a heat exchanger configured with the supplied arrangement.
    pub const fn new(arrangement: T) -> Self {
        Self(arrangement)
    }
}

impl<T: EffectivenessRelation> Hx<T> {
    /// Resolve outlet conditions for both streams using a known conductance and
    /// inlet states, returning a [`KnownConductanceAndInletsResult`].
    ///
    /// # Errors
    ///
    /// Returns an error if any of the supplied thermodynamic quantities violate
    /// their constraints (for example, a non-positive capacitance rate).
    pub fn known_conductance_and_inlets(
        &self,
        ua: ThermalConductance,
        inlets: [StreamInlet; 2],
    ) -> ConstraintResult<KnownConductanceResult> {
        functional::known_conductance_and_inlets(&self.0, ua, inlets)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use twine_thermo::HeatFlow;
    use uom::si::{
        f64::{ThermalConductance, ThermodynamicTemperature},
        power::kilowatt,
        ratio::ratio,
        thermal_conductance::kilowatt_per_kelvin,
        thermodynamic_temperature::degree_celsius,
    };

    use super::*;

    #[test]
    fn hx_usability() -> ConstraintResult<()> {
        let hx = Hx::new(arrangement::CounterFlow);

        let result = hx.known_conductance_and_inlets(
            ThermalConductance::new::<kilowatt_per_kelvin>(3. * 4.0_f64.ln()),
            [
                StreamInlet::new(
                    CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
                    ThermodynamicTemperature::new::<degree_celsius>(50.),
                ),
                StreamInlet::new(
                    CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
                    ThermodynamicTemperature::new::<degree_celsius>(80.),
                ),
            ],
        )?;

        let KnownConductanceResult {
            streams,
            effectiveness,
        } = result;

        assert_relative_eq!(effectiveness.get::<ratio>(), 2. / 3.);
        assert!(matches!(streams[0].heat_flow, HeatFlow::In(_)));
        assert!(matches!(streams[1].heat_flow, HeatFlow::Out(_)));
        for stream in streams {
            assert_relative_eq!(
                stream.heat_flow.signed().get::<kilowatt>().abs(),
                60.,
                max_relative = 1e-15
            );
            assert_relative_eq!(stream.outlet_temperature.get::<degree_celsius>(), 70.);
        }

        Ok(())
    }
}
