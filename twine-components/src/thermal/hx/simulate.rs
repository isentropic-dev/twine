use twine_core::constraint::ConstraintResult;
use twine_thermo::{HeatFlow, units::TemperatureDifference};
use uom::{
    ConstZero,
    si::f64::{Power, ThermalConductance},
};

use crate::thermal::hx::{
    Effectiveness,
    arrangement::Arrangement,
    capacity_ratio::CapacityRatio,
    ntu::Ntu,
    stream::{Stream, StreamInlet},
};

/// Simulate heat exchanger performance given the conductance and inlet
/// conditions.
///
/// This uses the effectivenss-NTU method to determine the heat transfer between
/// the two streams, then performs energy balances on each stream to determine
/// their outlet temperatures.
///
/// This assumes the both working fluids have a constant specific heat capacity
/// as they pass through the heat exchanger.
///
/// # Example
///
/// ```rust
/// use crate::thermal::hx::{
///     Arrangement, CapacitanceRate, KnownConductanceAndInlets, StreamInlet,
/// };
/// use uom::si::{
///     f64::{ThermalConductance, ThermodynamicTemperature},
///     thermal_conductance::watt_per_kelvin,
///     thermodynamic_temperature::degree_celsius,
/// };
/// // Create a counter-flow heat exchanger to simulate.
/// let hx = KnownConductanceAndInlets(Arrangement::CounterFlow);
///
/// // Execute the simulation.
/// //
/// // The result will contain the effectiveness of the heat exchanger,  as well
/// // fully-resolved streams. Each stream contains its outlet temperature and
/// // the heat transferred to/from it.
/// let result = hx.call(
///     ThermalConductance::new::<watt_per_kelvin>(50.),
///     [
///         StreamInlet::new(
///             CapacitanceRate::new::<watt_per_kelvin>(10.)?,
///             ThermodynamicTemperature::new::<degree_celsius>(50.),
///         ),
///         StreamInlet::new(
///             CapacitanceRate::new::<watt_per_kelvin>(15.)?,
///             ThermodynamicTemperature::new::<degree_celsius>(40.),
///         ),
///     ],
/// )?;
/// ```
pub struct KnownConductanceAndInlets(Arrangement);

#[derive(Debug, Clone, Copy)]
pub struct KnownConductanceAndInletsResult {
    pub streams: [Stream; 2],
    pub effectiveness: Effectiveness,
}

impl KnownConductanceAndInlets {
    fn calculate_max_heat_flow(inlets: [StreamInlet; 2]) -> ConstraintResult<[Stream; 2]> {
        let min_capacitance_rate = inlets[0].capacitance_rate.min(*inlets[1].capacitance_rate);
        let max_heat_flow =
            min_capacitance_rate * inlets[0].temperature.minus(inlets[1].temperature);

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

    /// Run the simulation.
    ///
    /// Given the conductance of the heat exchanger and inlet conditions as
    /// [`StreamInlet`], the fully resolved [streams](Stream) and heat exchanger
    /// [effectiveness](Effectiveness) will be returned.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the provided inputs are not
    /// withing their expected bounds.
    pub fn call(
        &self,
        ua: ThermalConductance,
        inlets: [StreamInlet; 2],
    ) -> ConstraintResult<KnownConductanceAndInletsResult> {
        let streams_with_max_heat = Self::calculate_max_heat_flow(inlets)?;
        let capacitance_rates = [inlets[0].capacitance_rate, inlets[1].capacitance_rate];
        let effectiveness = self.0.effectiveness(
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
}

#[cfg(test)]
mod tests {
    use uom::si::{
        f64::ThermodynamicTemperature, thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::kelvin,
    };

    use crate::thermal::hx::capacitance_rate::CapacitanceRate;

    use super::*;

    #[test]
    fn a_thing() -> ConstraintResult<()> {
        let hx = KnownConductanceAndInlets(Arrangement::CounterFlow);

        let result = hx.call(
            ThermalConductance::new::<watt_per_kelvin>(50.),
            [
                StreamInlet::new(
                    CapacitanceRate::new::<watt_per_kelvin>(10.)?,
                    ThermodynamicTemperature::new::<kelvin>(300.),
                ),
                StreamInlet::new(
                    CapacitanceRate::new::<watt_per_kelvin>(20.)?,
                    ThermodynamicTemperature::new::<kelvin>(250.),
                ),
            ],
        )?;

        println!("{result:?}");

        Ok(())
    }
}
