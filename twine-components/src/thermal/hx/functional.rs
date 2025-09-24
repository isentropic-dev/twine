//! Functional helpers for common heat exchanger calculations.

use twine_core::constraint::ConstraintResult;
use twine_thermo::{HeatFlow, units::TemperatureDifference};
use uom::{
    ConstZero,
    si::f64::{Power, ThermalConductance},
};

use crate::thermal::hx::{
    CapacityRatio, Effectiveness, EffectivenessNtu, Ntu, StreamInlet, stream::Stream,
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
///     CapacitanceRate,
///     CounterFlow,
///     StreamInlet,
///     functional,
/// };
///
/// # fn main() -> ConstraintResult<()> {
/// let result = functional::known_conductance_and_inlets(
///     &CounterFlow,
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
    arrangement: &impl EffectivenessNtu,
    ua: ThermalConductance,
    inlets: [StreamInlet; 2],
) -> ConstraintResult<KnownConductanceAndInletsResult> {
    let streams_with_max_heat = calculate_max_heat_flow(inlets)?;
    let capacitance_rates = [inlets[0].capacitance_rate, inlets[1].capacitance_rate];
    let effectiveness = arrangement.effectiveness(
        Ntu::from_conductance_and_capacitance_rates(ua, capacitance_rates)?,
        CapacityRatio::from_capacitance_rates(capacitance_rates)?,
    );

    Ok(KnownConductanceAndInletsResult {
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
pub struct KnownConductanceAndInletsResult {
    /// Final state for each stream after traversing the exchanger (same order as the inputs).
    pub streams: [Stream; 2],
    /// Overall effectiveness computed for the scenario.
    pub effectiveness: Effectiveness,
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

        let KnownConductanceAndInletsResult {
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
