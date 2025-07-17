mod traits;

pub mod ideal_gas;
pub mod incompressible;

pub use traits::{
    ControlVolumeDynamics, FlowOperations, StateFrom, StateOperations, ThermodynamicProperties,
};
