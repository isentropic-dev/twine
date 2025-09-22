use twine_core::constraint::ConstraintError;
use twine_thermo::{HeatFlow, units::TemperatureDifference};
use uom::{
    ConstZero,
    si::{
        f64::{Power, Ratio, ThermalConductance},
        ratio::ratio,
    },
};

use crate::thermal::hx::stream::{Stream, StreamInlet};

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
enum Arrangement {
    CounterFlow,
}

#[derive(Debug, thiserror::Error)]
enum HxError {
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(&'static str),
    #[error(transparent)]
    Constraint(#[from] ConstraintError),
}

struct KnownConductance(Arrangement);

impl KnownConductance {
    fn ntu(ua: ThermalConductance, capacitance_rates: [ThermalConductance; 2]) -> Ratio {
        ua / capacitance_rates[0].min(capacitance_rates[1])
    }

    fn effectiveness(
        &self,
        ua: ThermalConductance,
        capacitance_rates: [ThermalConductance; 2],
    ) -> Result<Ratio, HxError> {
        if capacitance_rates.iter().any(|rate| rate.is_infinite()) {
            let ntu = Self::ntu(ua, capacitance_rates);
            Ok(Ratio::new::<ratio>(1. - (-ntu.get::<ratio>()).exp()))
        } else {
            match self.0 {
                Arrangement::CounterFlow => todo!(),
            }
        }
    }

    fn max_heat_flow(streams: [StreamInlet; 2]) -> Result<[Stream; 2], HxError> {
        let min_capacitance_rate = streams[0].capacitance_rate.min(streams[1].capacitance_rate);
        let max_heat_flow =
            min_capacitance_rate * streams[0].temperature.minus(streams[1].temperature);

        Ok(
            match max_heat_flow
                .partial_cmp(&Power::ZERO)
                .expect("heat flow should not be NaN")
            {
                std::cmp::Ordering::Less => [
                    streams[0].with_heat_flow(HeatFlow::incoming(max_heat_flow.abs())?),
                    streams[1].with_heat_flow(HeatFlow::outgoing(max_heat_flow.abs())?),
                ],
                std::cmp::Ordering::Equal => [
                    streams[0].with_heat_flow(HeatFlow::None),
                    streams[1].with_heat_flow(HeatFlow::None),
                ],
                std::cmp::Ordering::Greater => [
                    streams[0].with_heat_flow(HeatFlow::outgoing(max_heat_flow)?),
                    streams[1].with_heat_flow(HeatFlow::incoming(max_heat_flow)?),
                ],
            },
        )
    }

    fn call(
        &self,
        ua: ThermalConductance,
        streams: [StreamInlet; 2],
    ) -> Result<[Stream; 2], HxError> {
        let max_heat_results = Self::max_heat_flow(streams)?;
        let effectiveness = self.effectiveness(
            ua,
            [streams[0].capacitance_rate, streams[1].capacitance_rate],
        )?;

        Ok([
            streams[0].with_heat_flow(HeatFlow::from_signed(
                effectiveness * max_heat_results[0].heat_flow.signed(),
            )?),
            streams[1].with_heat_flow(HeatFlow::from_signed(
                effectiveness * max_heat_results[1].heat_flow.signed(),
            )?),
        ])
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{
        f64::{MassRate, SpecificHeatCapacity, ThermodynamicTemperature},
        mass_rate::kilogram_per_second,
        specific_heat_capacity::joule_per_kilogram_kelvin,
        thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::thermal::hx::stream::{CapacitanceRate, capacitance_rate};

    use super::*;

    #[test]
    fn a_thing() -> Result<(), HxError> {
        let hx = KnownConductance(Arrangement::CounterFlow);

        let streams = [
            StreamInlet::new(
                CapacitanceRate::new::<watt_per_kelvin>(f64::INFINITY),
                ThermodynamicTemperature::new::<degree_celsius>(50.),
            ),
            StreamInlet::new(
                capacitance_rate(
                    MassRate::new::<kilogram_per_second>(0.01),
                    SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(4184.),
                ),
                ThermodynamicTemperature::new::<degree_celsius>(40.),
            ),
        ];

        let ua = ThermalConductance::new::<watt_per_kelvin>(50.);

        let streams = hx.call(ua, streams)?;

        println!("{streams:?}");
        Ok(())
    }
}
