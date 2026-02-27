//! Reusable observers for the Twine framework.
//!
//! This crate provides [`Observer`] implementations and capability traits that
//! work across different solvers in the Twine ecosystem.
//!
//! # Modules
//!
//! - [`traits`] — Capability traits for cross-solver observers
//!   ([`HasResidual`], [`HasObjective`], [`CanStopEarly`], [`CanAssumeWorse`])
//!
//! # Features
//!
//! - `plot` — Enables [`PlotObserver`] for visualizing solver behavior via egui.
//!   This feature adds dependencies on `eframe` and `egui_plot`.
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
pub use plot::PlotObserver;
