#![warn(missing_docs)]

//! Tools for analyzing heat exchangers.
//!
//! These utilities provide standard effectiveness-NTU relationships, helpers
//! for sizing exchangers, and functional APIs for working directly with
//! thermodynamic primitives.

pub mod arrangement;
mod capacitance_rate;
mod capacity_ratio;
pub mod discretized;
mod effectiveness_ntu;
pub mod functional;
mod stream;

pub use capacitance_rate::CapacitanceRate;
pub use capacity_ratio::CapacityRatio;
pub use effectiveness_ntu::{Effectiveness, Ntu};
pub use stream::{Stream, StreamInlet};
use twine_core::constraint::ConstraintResult;
use uom::si::f64::ThermalConductance;

use crate::thermal::hx::{
    effectiveness_ntu::{EffectivenessRelation, NtuRelation},
    functional::{KnownConditionsResult, KnownConductanceResult},
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
    /// inlet states, returning a [`KnownConductanceResult`].
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

impl<T: NtuRelation> Hx<T> {
    /// Determine the required conductance (UA) for a heat exchanger given one
    /// inlet condition and one fully-resolved stream.
    ///
    /// This method solves the "inverse" heat exchanger problem: given one stream's
    /// inlet conditions and another stream's complete state (inlet, outlet, and heat
    /// flow), it calculates the required UA and NTU values.
    ///
    /// The tuple parameter accepts `(StreamInlet, Stream)` where:
    /// - The first element is the inlet condition for one stream
    /// - The second element is the fully-resolved state for the other stream
    ///
    /// The fully-resolved [`Stream`] can be constructed from either a known heat flow
    /// using [`Stream::new_from_heat_flow`] or a known outlet temperature using
    /// [`Stream::new_from_outlet_temperature`].
    ///
    /// Returns a [`KnownConditionsResult`] containing both resolved streams, the
    /// calculated UA, and NTU.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use twine_core::constraint::ConstraintResult;
    /// use uom::si::{
    ///     f64::{Power, ThermodynamicTemperature},
    ///     power::kilowatt,
    ///     thermodynamic_temperature::degree_celsius,
    ///     thermal_conductance::kilowatt_per_kelvin,
    /// };
    /// use twine_components::thermal::hx::{arrangement, CapacitanceRate, Hx, Stream, StreamInlet};
    /// use twine_thermo::HeatFlow;
    ///
    /// # fn main() -> ConstraintResult<()> {
    /// let hx = Hx::new(arrangement::CounterFlow);
    ///
    /// // Construct the fully-resolved stream from a known heat flow
    /// let result = hx.known_conditions_and_inlets((
    ///     StreamInlet::new(
    ///         CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
    ///         ThermodynamicTemperature::new::<degree_celsius>(50.),
    ///     ),
    ///     Stream::new_from_heat_flow(
    ///         CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
    ///         ThermodynamicTemperature::new::<degree_celsius>(80.),
    ///         HeatFlow::outgoing(Power::new::<kilowatt>(60.))?,
    ///     ),
    /// ))?;
    ///
    /// // Access the calculated UA and NTU
    /// let ua = result.ua;
    /// let ntu = result.ntu;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any of the supplied thermodynamic quantities violate
    /// their constraints (for example, a non-positive capacitance rate).
    pub fn known_conditions_and_inlets(
        &self,
        streams: (StreamInlet, Stream),
    ) -> ConstraintResult<KnownConditionsResult> {
        functional::known_conditions_and_inlets(&self.0, streams)
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
