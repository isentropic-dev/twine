mod component;
mod composed;
pub mod legacy;
mod twine;

#[cfg(feature = "macros")]
pub use twine_macros::compose;

pub use component::Component;
pub use composed::{Composed, Composition};
pub use twine::Twine;
