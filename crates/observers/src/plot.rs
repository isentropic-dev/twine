//! Plotting observer for visualizing solver behavior.
//!
//! See [`PlotObserver`] for usage.

/// An observer that collects trace data during solving and displays it via egui.
///
/// # Example
///
/// ```ignore
/// let observer = PlotObserver::new(|event| /* extract x */)
///     .trace("Temperature", |event| /* extract y */)
///     .trace("Pressure", |event| /* extract y */);
///
/// let solution = solver::solve(model, problem, config, &mut observer);
///
/// observer.show("Results");
/// ```
pub struct PlotObserver {
    // TODO: Implement
}
