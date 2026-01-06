//! Functional helpers for common heat exchanger calculations.

use twine_core::constraint::ConstraintResult;
use twine_thermo::{HeatFlow, units::TemperatureDifference};
use uom::{
    ConstZero,
    si::f64::{Power, ThermalConductance},
};

use crate::thermal::hx::{
    Effectiveness, Ntu, StreamInlet,
    effectiveness_ntu::{EffectivenessRelation, NtuRelation},
    stream::Stream,
};

/// Analyze a heat exchanger when its conductance and inlet conditions are
/// known.
///
/// Given the conductance of the heat exchanger and inlet conditions as
/// [`StreamInlet`], the fully resolved [streams](Stream) and heat exchanger
/// [effectiveness](Effectiveness) will be returned.
///
/// # Example
///
/// ```rust
/// # use twine_core::constraint::ConstraintResult;
/// use uom::si::{
///     f64::{ThermalConductance, ThermodynamicTemperature},
///     power::kilowatt,
///     ratio::ratio,
///     thermal_conductance::kilowatt_per_kelvin,
///     thermodynamic_temperature::degree_celsius,
/// };
/// use twine_components::thermal::hx::{
///     arrangement,
///     CapacitanceRate,
///     StreamInlet,
///     functional,
/// };
///
/// # fn main() -> ConstraintResult<()> {
/// let result = functional::known_conductance_and_inlets(
///     &arrangement::CounterFlow,
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
///
/// # Errors
///
/// Returns `Err` if any supplied quantity violates its constraints (for
/// example, a non-positive capacitance rate).
pub fn known_conductance_and_inlets(
    arrangement: &impl EffectivenessRelation,
    ua: ThermalConductance,
    inlets: [StreamInlet; 2],
) -> ConstraintResult<KnownConductanceResult> {
    let streams_with_max_heat = calculate_max_heat_flow(inlets)?;
    let capacitance_rates = [inlets[0].capacitance_rate, inlets[1].capacitance_rate];
    let effectiveness = arrangement.effectiveness(
        Ntu::from_conductance_and_capacitance_rates(ua, capacitance_rates)?,
        capacitance_rates,
    );

    Ok(KnownConductanceResult {
        streams: [
            inlets[0].with_heat_flow(HeatFlow::from_signed(
                *effectiveness * streams_with_max_heat[0].heat_flow.signed(),
            )?),
            inlets[1].with_heat_flow(HeatFlow::from_signed(
                *effectiveness * streams_with_max_heat[1].heat_flow.signed(),
            )?),
        ],
        effectiveness,
    })
}

/// Resolved exchanger state returned from [`known_conductance_and_inlets`].
#[derive(Debug, Clone, Copy)]
pub struct KnownConductanceResult {
    /// Final state for each stream after traversing the exchanger (same order as the inputs).
    pub streams: [Stream; 2],
    /// Overall effectiveness computed for the scenario.
    pub effectiveness: Effectiveness,
}

/// Determine the required conductance (UA) for a heat exchanger given one
/// inlet condition and one fully-resolved stream.
///
/// This function solves the "inverse" heat exchanger problem: given one stream's
/// inlet conditions and another stream's complete state (inlet, outlet, and heat
/// flow), it calculates the required UA and NTU values needed to achieve that
/// performance.
///
/// The fully-resolved [`Stream`] can be constructed from either a known heat flow
/// using [`Stream::new_from_heat_flow`] or a known outlet temperature using
/// [`Stream::new_from_outlet_temperature`].
///
/// # Example
///
/// ```rust
/// # use twine_core::constraint::ConstraintResult;
/// use uom::si::{
///     f64::{Power, ThermodynamicTemperature},
///     power::kilowatt,
///     thermal_conductance::kilowatt_per_kelvin,
///     thermodynamic_temperature::degree_celsius,
/// };
/// use twine_components::thermal::hx::{
///     arrangement,
///     CapacitanceRate,
///     Stream,
///     StreamInlet,
///     functional,
/// };
/// use twine_thermo::HeatFlow;
///
/// # fn main() -> ConstraintResult<()> {
/// let result = functional::known_conditions_and_inlets(
///     &arrangement::CounterFlow,
///     (
///         StreamInlet::new(
///             CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
///             ThermodynamicTemperature::new::<degree_celsius>(50.),
///         ),
///         Stream::new_from_heat_flow(
///             CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
///             ThermodynamicTemperature::new::<degree_celsius>(80.),
///             HeatFlow::outgoing(Power::new::<kilowatt>(60.))?,
///         ),
///     ),
/// )?;
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
/// Returns `Err` if any supplied quantity violates its constraints (for
/// example, a non-positive capacitance rate).
pub fn known_conditions_and_inlets(
    arrangement: &impl NtuRelation,
    streams: (StreamInlet, Stream),
) -> ConstraintResult<KnownConditionsResult> {
    let streams_with_max_heat = calculate_max_heat_flow([streams.0, streams.1.into()])?;
    let capacitance_rates = [streams.0.capacitance_rate, streams.1.capacitance_rate];

    // Doesn't matter which stream we use to get max heat flow. Magnitude is the same.
    let max_heat_flow = streams_with_max_heat[0].heat_flow.signed().abs();
    let effectiveness =
        Effectiveness::from_quantity(streams.1.heat_flow.signed().abs() / max_heat_flow)?;

    let ntu = arrangement.ntu(effectiveness, capacitance_rates);

    Ok(KnownConditionsResult {
        streams: [
            streams.0.with_heat_flow(HeatFlow::from_signed(
                *effectiveness * streams_with_max_heat[0].heat_flow.signed(),
            )?),
            streams.1,
        ],
        ua: *ntu * capacitance_rates[0].min(*capacitance_rates[1]),
        ntu,
    })
}

/// Resolved exchanger state returned from [`known_conditions_and_inlets`].
///
/// This result type represents the complete solution to the "inverse" heat exchanger
/// problem, where the required UA is calculated from known operating conditions. It
/// includes both the fully-resolved streams and the calculated thermal design parameters
/// (UA and NTU) needed to achieve the specified performance.
#[derive(Debug, Clone, Copy)]
pub struct KnownConditionsResult {
    /// Final state for each stream after traversing the exchanger (same order as the inputs).
    pub streams: [Stream; 2],
    /// Heat exchanger conductance (UA) required to achieve the specified conditions.
    pub ua: ThermalConductance,
    /// Number of Transfer Units (NTU) for the exchanger under these conditions.
    pub ntu: Ntu,
}

fn calculate_max_heat_flow(inlets: [StreamInlet; 2]) -> ConstraintResult<[Stream; 2]> {
    let min_capacitance_rate = inlets[0].capacitance_rate.min(*inlets[1].capacitance_rate);
    let max_heat_flow = min_capacitance_rate * inlets[0].temperature.minus(inlets[1].temperature);

    Ok(
        match max_heat_flow
            .partial_cmp(&Power::ZERO)
            .expect("heat flow should not be NaN")
        {
            std::cmp::Ordering::Less => [
                inlets[0].with_heat_flow(HeatFlow::incoming(max_heat_flow.abs())?),
                inlets[1].with_heat_flow(HeatFlow::outgoing(max_heat_flow.abs())?),
            ],
            std::cmp::Ordering::Equal => [
                inlets[0].with_heat_flow(HeatFlow::None),
                inlets[1].with_heat_flow(HeatFlow::None),
            ],
            std::cmp::Ordering::Greater => [
                inlets[0].with_heat_flow(HeatFlow::outgoing(max_heat_flow)?),
                inlets[1].with_heat_flow(HeatFlow::incoming(max_heat_flow)?),
            ],
        },
    )
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use uom::si::{
        f64::ThermodynamicTemperature, power::kilowatt, ratio::ratio,
        thermal_conductance::kilowatt_per_kelvin, thermodynamic_temperature::degree_celsius,
    };

    use crate::thermal::hx::{CapacitanceRate, arrangement::CounterFlow};

    use super::*;

    #[test]
    fn known_conductance_and_inlets() -> ConstraintResult<()> {
        let result = super::known_conductance_and_inlets(
            &CounterFlow,
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

    #[test]
    fn known_conditions_and_inlets() -> ConstraintResult<()> {
        let result = super::known_conditions_and_inlets(
            &CounterFlow,
            (
                StreamInlet::new(
                    CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
                    ThermodynamicTemperature::new::<degree_celsius>(50.),
                ),
                Stream::new_from_heat_flow(
                    CapacitanceRate::new::<kilowatt_per_kelvin>(6.)?,
                    ThermodynamicTemperature::new::<degree_celsius>(80.),
                    HeatFlow::outgoing(Power::new::<kilowatt>(60.))?,
                ),
            ),
        )?;

        let KnownConditionsResult { streams, ua, ntu } = result;

        assert_relative_eq!(ua.get::<kilowatt_per_kelvin>(), 3. * 4.0_f64.ln());
        assert_relative_eq!(ntu.get::<ratio>(), 4.0_f64.ln());
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
