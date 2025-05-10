mod positive_mass_rate;
mod temperature_difference;

use uom::{
    si::{Quantity, ISQ, SI},
    typenum::{N1, N2, P2, Z0},
};

pub use positive_mass_rate::{PositiveMassRate, PositiveMassRateError};
pub use temperature_difference::TemperatureDifference;

/// Specific gas constant, J/kg·K in SI.
pub type SpecificGasConstant = Quantity<ISQ<P2, Z0, N2, Z0, N1, Z0, Z0>, SI<f64>, f64>;

/// Specific enthalpy, J/kg in SI.
pub type SpecificEnthalpy = Quantity<ISQ<P2, Z0, N2, Z0, Z0, Z0, Z0>, SI<f64>, f64>;

/// Specific entropy, J/kg·K in SI.
pub type SpecificEntropy = Quantity<ISQ<P2, Z0, N2, Z0, N1, Z0, Z0>, SI<f64>, f64>;

/// Specific internal energy, J/kg in SI.
pub type SpecificInternalEnergy = Quantity<ISQ<P2, Z0, N2, Z0, Z0, Z0, Z0>, SI<f64>, f64>;
