//! The `transient` module provides a domain-agnostic framework for simulating
//! dynamic systems using discrete time steps.
//!
//! It defines a modular architecture that separates concerns between integration,
//! control, and system dynamics, making it suitable for a wide range of applications
//! including physical simulations, control systems, and state-based models.
//!
//! At its core is the [`Simulation`] type, which manages a timeline of
//! [`TimeStep`]s and coordinates system evolution through:
//!
//! - [`Integrator`]: Proposes the next input using simulation history and a time step.
//! - [`Controller`]: Optionally adjusts that input before evaluation.
//! - [`StatefulComponent`]: A component with internal state and time derivatives.
//!
//! The framework is designed to be extensible, supporting custom integration schemes,
//! reusable control logic, and physical units via the [`uom`] crate.
//!
//! # Example Workflow
//!
//! 1. Define a component that implements [`Component`] or [`StatefulComponent`].
//! 2. Choose an [`Integrator`] (e.g., [`ForwardEuler`]).
//! 3. Optionally implement a [`Controller`].
//! 4. Use [`Simulation`] to evolve the system over time.
//!
//! See trait docs for implementation details and example patterns.
pub mod integrators;
mod simulation;
mod traits;
mod types;

#[cfg(test)]
mod test_utils;

pub use simulation::Simulation;
pub use traits::{Controller, HasTimeDerivative, Integrator, StatefulComponent, Temporal};
pub use types::{TimeDerivativeOf, TimeStep};
