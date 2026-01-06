use twine_thermo::{HeatFlow, units::TemperatureDifference};
use uom::si::f64::ThermodynamicTemperature;

use crate::thermal::hx::capacitance_rate::CapacitanceRate;

/// Inlet state for a stream entering the heat exchanger.
///
/// Assumes the fluid's specific heat remains constant through the exchanger.
#[derive(Debug, Clone, Copy)]
pub struct StreamInlet {
    pub(crate) capacitance_rate: CapacitanceRate,
    pub(crate) temperature: ThermodynamicTemperature,
}

impl StreamInlet {
    /// Capture the inlet capacitance rate and temperature.
    #[must_use]
    pub fn new(capacitance_rate: CapacitanceRate, temperature: ThermodynamicTemperature) -> Self {
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
                            self.temperature + (heat_flow.into_inner() / *self.capacitance_rate)
                        }
                        HeatFlow::Out(heat_flow) => {
                            self.temperature - (heat_flow.into_inner() / *self.capacitance_rate)
                        }
                        HeatFlow::None => self.temperature,
                    }
                }
            },
        }
    }
}

/// A fully-resolved heat exchanger stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stream {
    /// Effective capacitance rate for the stream.
    pub capacitance_rate: CapacitanceRate,
    /// Temperature at the exchanger inlet.
    pub inlet_temperature: ThermodynamicTemperature,
    /// Temperature after the stream leaves the exchanger.
    ///
    /// When the capacitance rate tends to infinity this matches the inlet
    /// temperature.
    pub outlet_temperature: ThermodynamicTemperature,
    /// Net heat flow direction and magnitude for the stream.
    pub heat_flow: HeatFlow,
}

impl Stream {
    /// Construct a fully-resolved stream from a known heat flow.
    ///
    /// Given the stream's inlet temperature, capacitance rate, and heat flow,
    /// this calculates the outlet temperature using the energy balance:
    /// `Q = C * (T_out - T_in)`.
    ///
    /// This constructor is useful when you know the heat transfer rate for a stream
    /// (for example, from measurements or system specifications) and need to determine
    /// the resulting outlet temperature.
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
    /// use twine_components::thermal::hx::{CapacitanceRate, Stream};
    /// use twine_thermo::HeatFlow;
    ///
    /// # fn main() -> ConstraintResult<()> {
    /// let stream = Stream::new_from_heat_flow(
    ///     CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
    ///     ThermodynamicTemperature::new::<degree_celsius>(80.),
    ///     HeatFlow::outgoing(Power::new::<kilowatt>(60.))?,
    /// );
    ///
    /// // Outlet temperature is calculated automatically
    /// assert_eq!(
    ///     stream.outlet_temperature.get::<degree_celsius>(),
    ///     60.0
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new_from_heat_flow(
        capacitance_rate: CapacitanceRate,
        inlet_temperature: ThermodynamicTemperature,
        heat_flow: HeatFlow,
    ) -> Self {
        Self {
            capacitance_rate,
            inlet_temperature,
            outlet_temperature: match heat_flow {
                HeatFlow::In(heat_rate) => {
                    inlet_temperature + heat_rate.into_inner() / *capacitance_rate
                }
                HeatFlow::Out(heat_rate) => {
                    inlet_temperature - heat_rate.into_inner() / *capacitance_rate
                }
                HeatFlow::None => inlet_temperature,
            },
            heat_flow,
        }
    }

    /// Construct a fully-resolved stream from known inlet and outlet temperatures.
    ///
    /// Given the stream's inlet and outlet temperatures along with its capacitance rate,
    /// this calculates the heat flow using the energy balance: `Q = C * (T_out - T_in)`.
    ///
    /// The heat flow direction is automatically determined from the temperature change:
    /// - If outlet > inlet, the heat flow is [`HeatFlow::In`]
    /// - If outlet < inlet, the heat flow is [`HeatFlow::Out`]
    /// - If outlet = inlet, the heat flow is [`HeatFlow::None`]
    ///
    /// This constructor is useful when you know both inlet and outlet temperatures for
    /// a stream (for example, from measurements) and need to determine the heat transfer
    /// rate.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use twine_core::constraint::ConstraintResult;
    /// use uom::si::{
    ///     f64::ThermodynamicTemperature,
    ///     power::kilowatt,
    ///     thermal_conductance::kilowatt_per_kelvin,
    ///     thermodynamic_temperature::degree_celsius,
    /// };
    /// use twine_components::thermal::hx::{CapacitanceRate, Stream};
    /// use twine_thermo::HeatFlow;
    ///
    /// # fn main() -> ConstraintResult<()> {
    /// let stream = Stream::new_from_outlet_temperature(
    ///     CapacitanceRate::new::<kilowatt_per_kelvin>(3.)?,
    ///     ThermodynamicTemperature::new::<degree_celsius>(80.),
    ///     ThermodynamicTemperature::new::<degree_celsius>(60.),
    /// );
    ///
    /// // Heat flow magnitude is calculated automatically (80°C → 60°C means heat is leaving)
    /// assert!(matches!(stream.heat_flow, HeatFlow::Out(_)));
    /// assert_eq!(
    ///     stream.heat_flow.signed().abs().get::<kilowatt>(),
    ///     60.0
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the temperatures cannot be compared (e.g., contain NaN values) or if
    /// the calculated heat rate magnitude is invalid (which should not occur in normal use).
    #[must_use]
    pub fn new_from_outlet_temperature(
        capacitance_rate: CapacitanceRate,
        inlet_temperature: ThermodynamicTemperature,
        outlet_temperature: ThermodynamicTemperature,
    ) -> Self {
        let heat_rate_magnitude =
            *capacitance_rate * inlet_temperature.minus(outlet_temperature).abs();

        Self {
            capacitance_rate,
            inlet_temperature,
            outlet_temperature,
            heat_flow: match inlet_temperature
                .partial_cmp(&outlet_temperature)
                .expect("temperatures to be comparable")
            {
                std::cmp::Ordering::Less => HeatFlow::incoming(heat_rate_magnitude)
                    .expect("heat rate magnitude should always be positive"),
                std::cmp::Ordering::Equal => HeatFlow::None,
                std::cmp::Ordering::Greater => HeatFlow::outgoing(heat_rate_magnitude)
                    .expect("heat rate magnitude should always be positive"),
            },
        }
    }
}

impl From<Stream> for StreamInlet {
    fn from(stream: Stream) -> Self {
        Self {
            capacitance_rate: stream.capacitance_rate,
            temperature: stream.inlet_temperature,
        }
    }
}

#[cfg(test)]
mod tests {
    use twine_core::constraint::ConstraintResult;
    use uom::si::{
        f64::Power, power::watt, thermal_conductance::watt_per_kelvin,
        thermodynamic_temperature::kelvin,
    };

    use super::*;

    #[test]
    fn stream_inlet_with_heat_flow() -> ConstraintResult<()> {
        let capacitance_rate = CapacitanceRate::new::<watt_per_kelvin>(10.)?;
        let inlet_temperature = ThermodynamicTemperature::new::<kelvin>(300.);
        let heat_rate = Power::new::<watt>(20.);

        let inlet = StreamInlet::new(capacitance_rate, inlet_temperature);

        let no_heat = inlet.with_heat_flow(HeatFlow::None);
        let incoming = inlet.with_heat_flow(HeatFlow::incoming(heat_rate)?);
        let outgoing = inlet.with_heat_flow(HeatFlow::outgoing(heat_rate)?);

        assert_eq!(
            no_heat,
            Stream {
                capacitance_rate,
                inlet_temperature,
                outlet_temperature: inlet_temperature,
                heat_flow: HeatFlow::None
            }
        );
        assert_eq!(
            incoming,
            Stream {
                capacitance_rate,
                inlet_temperature,
                outlet_temperature: ThermodynamicTemperature::new::<kelvin>(302.),
                heat_flow: HeatFlow::incoming(heat_rate)?
            }
        );
        assert_eq!(
            outgoing,
            Stream {
                capacitance_rate,
                inlet_temperature,
                outlet_temperature: ThermodynamicTemperature::new::<kelvin>(298.),
                heat_flow: HeatFlow::outgoing(heat_rate)?
            }
        );

        Ok(())
    }

    #[test]
    fn stream_new_from_heat_rate() -> ConstraintResult<()> {
        let capacitance_rate = CapacitanceRate::new::<watt_per_kelvin>(10.)?;
        let inlet_temperature = ThermodynamicTemperature::new::<kelvin>(300.);
        let heat_flow = HeatFlow::incoming(Power::new::<watt>(20.))?;

        let stream = Stream::new_from_heat_flow(capacitance_rate, inlet_temperature, heat_flow);

        assert_eq!(
            stream,
            Stream {
                capacitance_rate,
                inlet_temperature,
                outlet_temperature: ThermodynamicTemperature::new::<kelvin>(302.),
                heat_flow
            }
        );

        Ok(())
    }

    #[test]
    fn stream_new_from_outlet_temperature() -> ConstraintResult<()> {
        let capacitance_rate = CapacitanceRate::new::<watt_per_kelvin>(10.)?;
        let inlet_temperature = ThermodynamicTemperature::new::<kelvin>(300.);
        let outlet_temperature = ThermodynamicTemperature::new::<kelvin>(302.);

        let stream = Stream::new_from_outlet_temperature(
            capacitance_rate,
            inlet_temperature,
            outlet_temperature,
        );

        assert_eq!(
            stream,
            Stream {
                capacitance_rate,
                inlet_temperature,
                outlet_temperature,
                heat_flow: HeatFlow::incoming(Power::new::<watt>(20.))?
            }
        );

        Ok(())
    }
}
