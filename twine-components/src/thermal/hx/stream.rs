use twine_core::constraint::{Constrained, StrictlyPositive};
use twine_thermo::HeatFlow;
use uom::si::f64::{MassRate, SpecificHeatCapacity, ThermalConductance, ThermodynamicTemperature};

pub(crate) type CapacitanceRate = ThermalConductance;

pub(crate) fn capacitance_rate(
    mass_rate: MassRate,
    specific_heat: SpecificHeatCapacity,
) -> CapacitanceRate {
    mass_rate * specific_heat
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StreamInlet {
    pub(crate) capacitance_rate: CapacitanceRate,
    pub(crate) temperature: ThermodynamicTemperature,
}

impl StreamInlet {
    pub(crate) fn new(
        capacitance_rate: ThermalConductance,
        temperature: ThermodynamicTemperature,
    ) -> Self {
        Self {
            capacitance_rate,
            temperature,
        }
    }

    pub(crate) fn with_heat_flow(self, heat_flow: HeatFlow) -> Stream {
        Stream {
            capacitance_rate: self.capacitance_rate,
            inlet_temperature: self.temperature,
            heat_flow,
            outlet_temperature: {
                if self.capacitance_rate.is_infinite() {
                    self.temperature
                } else {
                    match heat_flow {
                        HeatFlow::In(heat_flow) => {
                            self.temperature + (heat_flow.into_inner() / self.capacitance_rate)
                        }
                        HeatFlow::Out(heat_flow) => {
                            self.temperature - (heat_flow.into_inner() / self.capacitance_rate)
                        }
                        HeatFlow::None => self.temperature,
                    }
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Stream {
    pub(crate) capacitance_rate: ThermalConductance,
    inlet_temperature: ThermodynamicTemperature,
    outlet_temperature: ThermodynamicTemperature,
    pub(crate) heat_flow: HeatFlow,
}

impl Stream {
    fn is_source(&self) -> bool {
        matches!(self.heat_flow, HeatFlow::Out(_))
    }

    fn is_sink(&self) -> bool {
        matches!(self.heat_flow, HeatFlow::In(_))
    }
}

// #[derive(Debug, Clone, Copy)]
// pub(crate) struct Stream {
//     mass_rate: MassRate,
//     specific_heat: SpecificHeatCapacity,
//     inlet_temperature: ThermodynamicTemperature,
// }

// impl Stream {
//     pub(crate) fn capacitance_rate(&self) -> ThermalConductance {
//         self.mass_rate * self.specific_heat
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub(crate) enum HxPort {
//     Stream(Stream),
//     IsoThermal(ThermodynamicTemperature),
// }

// impl HxPort {
//     pub(crate) fn stream(
//         mass_rate: MassRate,
//         specific_heat: SpecificHeatCapacity,
//         inlet_temperature: ThermodynamicTemperature,
//     ) -> Self {
//         Self::Stream(Stream {
//             mass_rate,
//             specific_heat,
//             inlet_temperature,
//         })
//     }

//     pub(crate) fn isothermal(temperature: ThermodynamicTemperature) -> Self {
//         Self::IsoThermal(temperature)
//     }

//     pub(crate) fn temperature(&self) -> ThermodynamicTemperature {
//         match *self {
//             HxPort::Stream(stream) => stream.inlet_temperature,
//             HxPort::IsoThermal(temperature) => temperature,
//         }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub(crate) struct HxPortResult {
//     pub(crate) port: HxPort,
//     pub(crate) heat_flow: HeatFlow,
// }

// impl HxPortResult {
//     pub(crate) fn is_source(&self) -> bool {
//         matches!(self.heat_flow, HeatFlow::Out(_))
//     }

//     pub(crate) fn is_sink(&self) -> bool {
//         matches!(self.heat_flow, HeatFlow::In(_))
//     }

//     pub(crate) fn outlet_temperature(&self) -> ThermodynamicTemperature {
//         match (self.port, self.heat_flow) {
//             (HxPort::Stream(stream), HeatFlow::In(heat_flow)) => {
//                 stream.inlet_temperature + (heat_flow.into_inner() / stream.capacitance_rate())
//             }
//             (HxPort::Stream(stream), HeatFlow::Out(heat_flow)) => {
//                 stream.inlet_temperature - (heat_flow.into_inner() / stream.capacitance_rate())
//             }
//             (HxPort::Stream(stream), HeatFlow::None) => stream.inlet_temperature,
//             (HxPort::IsoThermal(temperature), _) => temperature,
//         }
//     }
// }

// pub(crate) struct HxResult {
//     pub(crate) heat_flow: Power,
// }

// #[derive(Debug, Clone, Copy)]
// pub(crate) struct HxHeatFlow {
//     pub(crate) source: HxPort,
//     pub(crate) sink: HxPort,
//     pub(crate) value: Power,
// }

// impl HxHeatFlow {
//     pub(crate) fn source_outlet_temperature(&self) -> ThermodynamicTemperature {
//         match self.source {
//             HxPort::Stream(stream) => {
//                 stream.inlet_temperature - (self.value / stream.capacitance_rate())
//             }
//             HxPort::IsoThermal(temperature) => temperature,
//         }
//     }

//     pub(crate) fn sink_outlet_temperature(&self) -> ThermodynamicTemperature {
//         match self.sink {
//             HxPort::Stream(stream) => {
//                 stream.inlet_temperature + (self.value / stream.capacitance_rate())
//             }
//             HxPort::IsoThermal(temperature) => temperature,
//         }
//     }
// }
