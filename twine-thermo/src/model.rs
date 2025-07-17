mod traits;

pub mod ideal_gas;
pub mod incompressible;

pub use traits::{
    ControlVolumeConstantPressure, ControlVolumeDynamics, ControlVolumeFixedFlow, FlowOperations,
    StateFrom, StateOperations, ThermodynamicProperties,
};
