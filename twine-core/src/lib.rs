#[cfg(feature = "macros")]
pub use twine_macros::compose;

mod callable;
mod legacy;
mod twine;

pub use callable::Callable;
pub use legacy::Component;
pub use twine::{Then, Twine};
