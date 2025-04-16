pub mod error;
pub mod extrapolate;
pub mod one;
pub mod two;

pub use extrapolate::Extrapolate;
pub use one::{Interp1D, Strategy1D};
pub use two::{Interp2D, Strategy2D};
