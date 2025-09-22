use twine_core::constraint::ConstraintResult;
use twine_thermo::{HeatFlow, units::TemperatureDifference};
use uom::{
    ConstZero,
    si::f64::{Power, ThermalConductance},
};

use crate::thermal::hx::{
    arrangement::Arrangement,
    capacity_ratio::CapacityRatio,
    ntu::Ntu,
    stream::{Stream, StreamInlet},
};

pub struct KnownConductanceAndInlets(Arrangement);

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

    pub fn call(
        &self,
        ua: ThermalConductance,
        inlets: [StreamInlet; 2],
    ) -> ConstraintResult<[Stream; 2]> {
        let streams_with_max_heat = Self::calculate_max_heat_flow(inlets)?;
        let capacitance_rates = [inlets[0].capacitance_rate, inlets[1].capacitance_rate];
        let effectiveness = self.0.effectiveness(
            Ntu::from_conductance_and_capacitance_rates(ua, capacitance_rates)?,
            CapacityRatio::from_capacitance_rates(capacitance_rates)?,
        );

        Ok([
            inlets[0].with_heat_flow(HeatFlow::from_signed(
                effectiveness * streams_with_max_heat[0].heat_flow.signed(),
            )?),
            inlets[1].with_heat_flow(HeatFlow::from_signed(
                effectiveness * streams_with_max_heat[1].heat_flow.signed(),
            )?),
        ])
    }
}

#[cfg(test)]
mod tests {
    use uom::si::{
        f64::ThermodynamicTemperature, thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::degree_celsius,
    };

    use crate::thermal::hx::capacitance_rate::CapacitanceRate;

    use super::*;

    #[test]
    fn a_thing() -> ConstraintResult<()> {
        let hx = KnownConductanceAndInlets(Arrangement::CounterFlow);

        let streams = hx.call(
            ThermalConductance::new::<watt_per_kelvin>(50.),
            [
                StreamInlet::new(
                    CapacitanceRate::new::<watt_per_kelvin>(10.)?,
                    ThermodynamicTemperature::new::<degree_celsius>(50.),
                ),
                StreamInlet::new(
                    CapacitanceRate::new::<watt_per_kelvin>(15.)?,
                    ThermodynamicTemperature::new::<degree_celsius>(40.),
                ),
            ],
        )?;

        println!("{:?}", streams[0]);
        println!("{:?}", streams[1]);

        Ok(())
    }
}
