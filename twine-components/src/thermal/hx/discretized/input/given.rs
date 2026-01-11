use uom::si::f64::ThermodynamicTemperature;

use crate::thermal::hx::discretized::HeatTransferRate;

/// Specifies the additional constraint needed to fully define the heat exchanger.
///
/// Given the inlet states, pressure drops, and mass flows, one of these variants
/// is required to solve the energy balance and determine the outlet states.
///
/// The "top" and "bottom" labels refer to the physical stream assignment,
/// not necessarily the hot/cold side of the heat exchanger.
#[derive(Debug, Clone, Copy)]
pub enum Given {
    /// Specify the top stream outlet temperature.
    TopOutletTemp(ThermodynamicTemperature),

    /// Specify the bottom stream outlet temperature.
    BottomOutletTemp(ThermodynamicTemperature),

    /// Specify the heat transfer rate.
    ///
    /// The direction is encoded in the [`HeatTransferRate`] variant.
    HeatTransferRate(HeatTransferRate),
}
