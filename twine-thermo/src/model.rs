mod traits;

pub mod ideal_gas;
pub mod incompressible;

#[cfg(feature = "coolprop")]
pub mod coolprop;

pub use traits::{StateFrom, ThermodynamicProperties};
