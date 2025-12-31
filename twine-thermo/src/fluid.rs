//! Canonical fluid identifiers.
//!
//! A fluid type names a substance, and each model defines how that name is
//! interpreted, often via trait implementations (e.g., constants for idealized
//! models like [`PerfectGas`](crate::model::PerfectGas) or backend identifiers
//! for external property libraries like [`CoolProp`](crate::model::CoolProp)).
//!
//! Some fluids are simple unit-like types, while others carry state-defining data.

mod air;
mod carbon_dioxide;
mod water;

pub use air::Air;
pub use carbon_dioxide::CarbonDioxide;
pub use water::Water;
