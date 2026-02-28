//! Reusable observers for the Twine framework.
//!
//! This crate provides [`Observer`] implementations and capability traits that
//! work across different solvers in the Twine ecosystem.
//!
//! # Crate position in the dependency graph
//!
//! `twine-observers` sits at the top of the stack:
//!
//! ```text
//! twine-core  ←  twine-solvers  ←  twine-observers
//! ```
//!
//! This is intentional. Observers know about solvers — they implement capability
//! traits (see [`traits`]) for the concrete event and action types that solvers
//! expose. Solvers know nothing about observers. Removing this crate leaves
//! `twine-solvers` entirely unaffected.
//!
//! # Modules
//!
//! - [`traits`] — Capability traits for cross-solver observers
//!   ([`HasResidual`], [`HasObjective`], [`CanStopEarly`], [`CanAssumeWorse`])
//!
//! # Features
//!
//! - `plot` — Enables [`PlotObserver`] and [`ShowConfig`] for visualizing solver
//!   behavior via egui. This feature adds dependencies on `eframe` and `egui_plot`.
//!
//! [`Observer`]: twine_core::Observer
//! [`HasResidual`]: traits::HasResidual
//! [`HasObjective`]: traits::HasObjective
//! [`CanStopEarly`]: traits::CanStopEarly
//! [`CanAssumeWorse`]: traits::CanAssumeWorse

pub mod traits;

#[cfg(feature = "plot")]
mod plot;

#[cfg(feature = "plot")]
pub use plot::{PlotObserver, ShowConfig};
