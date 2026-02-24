//! Reusable observers for the Twine framework.
//!
//! This crate provides [`Observer`] implementations that work across different
//! solvers in the Twine ecosystem.
//!
//! # Features
//!
//! - `plot` — Enables [`PlotObserver`] for visualizing solver behavior via egui.
//!   This feature adds dependencies on `eframe` and `egui_plot`.
//!
//! [`Observer`]: twine_core::Observer

#[cfg(feature = "plot")]
mod plot;

#[cfg(feature = "plot")]
pub use plot::PlotObserver;
