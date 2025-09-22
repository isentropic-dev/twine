use twine_thermo::HeatFlow;
use uom::si::f64::ThermodynamicTemperature;

use crate::thermal::hx::capacitance_rate::CapacitanceRate;

#[derive(Debug, Clone, Copy)]
pub struct StreamInlet {
    pub(crate) capacitance_rate: CapacitanceRate,
    pub(crate) temperature: ThermodynamicTemperature,
}

impl StreamInlet {
    pub(crate) fn new(
        capacitance_rate: CapacitanceRate,
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
    pub(crate) capacitance_rate: CapacitanceRate,
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
