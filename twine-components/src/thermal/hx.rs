#![warn(missing_docs)]

//! Tools for analyzing heat exchangers.
//!
//! This includes functions for:
//! - Calculating effectiveness/NTU for different arrangements (i.e. counter-flow)
//! - Simulating heat exchanger performance
//! - Sizing heat exchangers given a desired performance

mod arrangement;
mod capacitance_rate;
mod capacity_ratio;
mod effectiveness;
mod ntu;
mod scenario;
mod stream;

pub use arrangement::CounterFlow;
pub use capacitance_rate::CapacitanceRate;
pub use capacity_ratio::CapacityRatio;
pub use effectiveness::Effectiveness;
pub use ntu::Ntu;
pub use scenario::Scenario;
pub use stream::StreamInlet;
use twine_core::constraint::ConstraintResult;

use crate::thermal::hx::{
    arrangement::Arrangement, scenario::known_conductance_and_inlets::known_conductance_and_inlets,
    stream::Stream,
};

/// Analyze heat exchanger performance in various scenarios.
///
/// This assumes that both working fluids have a constant specific heat capacity
/// as they pass through the heat exchanger.
///
/// # Example
///
/// ```rust
/// # use twine_core::constraint::ConstraintError;
/// use uom::si::{
///     f64::{ThermalConductance, ThermodynamicTemperature},
///     thermal_conductance::kilowatt_per_kelvin,
///     thermodynamic_temperature::degree_celsius,
/// };
///
/// use twine_components::thermal::hx::{
///     CapacitanceRate, CounterFlow, HxResult, Scenario, StreamInlet, hx,
/// };
///
/// let result = hx(
///     &CounterFlow,
///     Scenario::KnownConductanceAndInlets {
///         ua: ThermalConductance::new::<kilowatt_per_kelvin>(3. * 4.0_f64.ln()),
///         inlets: [
///             StreamInlet::new(
///                 CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
///                 ThermodynamicTemperature::new::<degree_celsius>(50.),
///             ),
///             StreamInlet::new(
///                 CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
///                 ThermodynamicTemperature::new::<degree_celsius>(80.),
///             ),
///         ],
///     },
/// )?;
/// # Ok::<(), ConstraintError>(())
/// ```
///
/// # Errors
///
/// This function will return an error if any of the provided inputs are not
/// within their expected bounds.
pub fn hx(arrangement: &impl Arrangement, scenario: Scenario) -> ConstraintResult<HxResult> {
    match scenario {
        Scenario::KnownConductanceAndInlets { ua, inlets } => {
            known_conductance_and_inlets(arrangement, ua, inlets)
        }
    }
}

/// Result of calling the heat exchanger component.
#[derive(Debug, Clone, Copy)]
pub struct HxResult {
    /// The fully-resolved heat exchanger streams.
    pub streams: [Stream; 2],
    /// The effectiveness of the heat exchanger.
    pub effectiveness: Effectiveness,
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
        let result = hx(
            &CounterFlow,
            Scenario::KnownConductanceAndInlets {
                ua: ThermalConductance::new::<kilowatt_per_kelvin>(3. * 4.0_f64.ln()),
                inlets: [
                    StreamInlet::new(
                        CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
                        ThermodynamicTemperature::new::<degree_celsius>(50.),
                    ),
                    StreamInlet::new(
                        CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
                        ThermodynamicTemperature::new::<degree_celsius>(80.),
                    ),
                ],
            },
        )?;

        let HxResult {
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
