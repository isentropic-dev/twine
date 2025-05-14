//! A framework for simulating dynamic systems over discrete time steps.
//!
//! The `transient` module provides composable tools for evolving time-based systems.
//! It separates numerical integration, control policy, and system dynamics into
//! well-defined roles, enabling reusable simulation strategies.
//!
//! # Core Roles
//!
//! - [`Simulation`]: Maintains a timeline of [`TimeStep`]s and exposes the component.
//!   It is passive and used by other tools to evolve the system.
//! - [`Integrator`]: Proposes a new input, typically using simulation history
//!   and a time step.
//! - [`Controller`]: Adjusts the integrator-proposed input, evaluates the
//!   component, and records the result on the [`Simulation`].
//!   This is the active driver of the simulation loop.
//! - [`StatefulComponent`]: A [`Component`] whose input encodes system state and
//!   whose output provides time derivatives.
//!
//! # Stepping the System
//!
//! A simulation advances by calling [`Controller::step`], which:
//!
//! 1. Delegates to an [`Integrator`] to propose the next input.
//! 2. Optionally adjusts the input.
//! 3. Evaluates the component.
//! 4. Records the result as a new [`TimeStep`].
//!
//! Here’s how to advance a simulation by one step:
//!
//! ```ignore
//! let controller = SomeController;
//! let integrator = ForwardEuler;
//! let mut sim = Simulation::new(component, initial_input)?;
//! controller.step(&mut sim, &integrator, dt)?;
//! ```
//!
//! The unit type `()` implements [`Controller`] as a no-op, making it a
//! convenient default when no control logic is needed:
//!
//! ```ignore
//! ().step(&mut sim, &integrator, dt)?;
//! ```
//!
//! # Extensibility
//!
//! The framework supports custom integration schemes, reusable control logic,
//! and unit-aware state modeling via the [`uom`] crate.
//!
//! You can:
//! - Implement new [`Integrator`] strategies (e.g., RK4, implicit).
//! - Apply domain-specific control (e.g., feedback, constraint enforcement).
//! - Switch between controllers or integrators during a [`Simulation`] to
//!   reflect changes in system behavior.
//!
//! See trait docs for implementation details and extension patterns.
pub mod integrators;
mod simulation;
mod traits;
mod types;

#[cfg(test)]
mod test_utils;

pub use simulation::Simulation;
pub use traits::{Controller, HasTimeDerivative, Integrator, StatefulComponent, Temporal};
pub use types::{StepError, TimeDerivativeOf, TimeStep};
