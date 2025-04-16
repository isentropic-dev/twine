pub mod error;
pub mod extrapolate;
pub mod n;
pub mod one;
pub mod three;
pub mod two;

pub use extrapolate::Extrapolate;
pub use n::{InterpND, StrategyND};
pub use one::{Interp1D, Strategy1D};
pub use three::{Interp3D, Strategy3D};
pub use two::{Interp2D, Strategy2D};
