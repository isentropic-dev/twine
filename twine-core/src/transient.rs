//! A framework for simulating dynamic systems over discrete time steps.
//!
//! The `transient` module provides composable tools for evolving time-based systems.
//! It separates system dynamics, numerical integration, and control policy into
//! distinct roles, enabling flexible and reusable simulation strategies.
//!
//! # Core Concepts
//!
//! - [`Simulation`]: Owns a [`Component`] and its history of [`TimeStep`]s,
//!   and drives the simulation forward by stepping the system through time.
//! - [`Integrator`]: Proposes the next input value based on a time increment
//!   and the current simulation history.
//! - [`Controller`]: Adjusts the integrator’s proposed input, enabling the
//!   application of constraints, feedback mechanisms, or custom control logic.
//! - [`StatefulComponent`]: A [`Component`] whose input encodes the system state,
//!   and whose output yields time derivatives.
//!
//! # Advancing the Simulation
//!
//! Simulations advance by calling [`Simulation::step`], which:
//!
//! 1. Delegates to an [`Integrator`] to propose the next input.
//! 2. Adjusts the proposed input via a [`Controller`].
//! 3. Evaluates the component with the adjusted input.
//! 4. Records the result as the latest [`TimeStep`].
//!
//! Example of stepping a simulation forward:
//!
//! ```ignore
//! let mut sim = Simulation::new(component, initial_input)?;
//! let controller = SomeController;
//! let integrator = ForwardEuler;
//! sim.step(dt, &integrator, &controller)?;
//! ```
//!
//! For simulations where no state integration is required, use [`AdvanceTime`]
//! as the integrator.
//! If no control logic is needed, the [`PassThrough`] controller passes inputs
//! through unchanged.
//! Together, they provide the simplest possible time advancement for a simulation.
//!
//! # Extensibility
//!
//! Designed for extensibility, the framework supports:
//!
//! - Custom integration strategies (e.g., RK4, implicit methods).
//! - Modular, reusable control logic tailored to domain-specific needs.
//! - Swapping integrators and controllers at each simulation step to reflect
//!   changes in system behavior or requirements.
//! - Unit-aware modeling using the [`uom`] crate.
//!
//! See individual trait and type documentation for more examples and
//! implementation details.
pub mod controllers;
pub mod integrators;
mod simulation;
mod traits;
mod types;

#[cfg(test)]
mod test_utils;

pub use simulation::{Simulation, StepError, Stepping};
pub use traits::{Controller, Integrator, StatefulComponent, Temporal};
pub use types::{TimeDerivativeOf, TimeIncrement, TimeIncrementError, TimeStep};
