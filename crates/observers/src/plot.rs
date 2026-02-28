//! Plotting observer for visualizing solver behavior.
//!
//! See [`PlotObserver`] and [`Plottable`] for usage.

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use twine_core::Observer;

/// Configuration for rendering a [`PlotObserver`] result.
///
/// Construct with [`ShowConfig::new`] and chain builder methods as needed.
/// All fields are independent with sensible defaults.
///
/// # Example
///
/// ```ignore
/// obs.show(ShowConfig::new().title("Bisection").legend().log_y())?;
/// ```
pub struct ShowConfig {
    title: Option<String>,
    legend: bool,
    log_y: bool,
}

impl ShowConfig {
    /// Creates a new `ShowConfig` with defaults: no title, no legend, linear scale.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            legend: false,
            log_y: false,
        }
    }

    /// Sets the window title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Enables a legend labeling each trace by name.
    #[must_use]
    pub fn legend(mut self) -> Self {
        self.legend = true;
        self
    }

    /// Enables a logarithmic y-axis (base 10).
    ///
    /// y values are transformed with log₁₀ before plotting. Non-positive
    /// values are silently skipped.
    #[must_use]
    pub fn log_y(mut self) -> Self {
        self.log_y = true;
        self
    }
}

impl Default for ShowConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts plottable data from a solver event.
///
/// Implement this on your event type to use [`PlotObserver`] directly as a
/// solver observer (the "direct path"). Return `None` from [`x`][Plottable::x]
/// to skip the event entirely; return `None` in a trace slot to skip that
/// trace for the event.
///
/// This trait is most useful when your event type is defined in your own crate,
/// which lets you satisfy the orphan rule. For standard library event types
/// (e.g. `bisection::Event`), use the closure path via
/// [`PlotObserver::record`] instead.
///
/// # Example — direct path with a local event type
///
/// ```ignore
/// // In your crate, where MyEvent is local:
/// impl Plottable<2> for MyEvent {
///     fn x(&self) -> Option<f64> {
///         Some(self.iteration as f64)
///     }
///
///     fn traces(&self) -> [Option<f64>; 2] {
///         [Some(self.residual), self.step_size]
///     }
/// }
///
/// let mut obs = PlotObserver::<2>::new(["Residual", "Step size"]);
/// my_solver::solve(&model, &problem, config, &mut obs)?;
/// obs.show(ShowConfig::new().title("My solver").legend())?;
/// ```
pub trait Plottable<const N: usize> {
    /// The x-axis value for this event, or `None` to skip recording entirely.
    fn x(&self) -> Option<f64>;

    /// The y-axis values for each trace.
    ///
    /// `None` in a slot skips that trace for this event while leaving others
    /// unaffected.
    fn traces(&self) -> [Option<f64>; N];
}

/// An observer that collects trace data during solving and displays it via egui.
///
/// The const generic `N` is the number of traces. Create with
/// [`PlotObserver::new`], passing the trace names. Record data by either:
///
/// - **Direct path** — Implement [`Plottable<N>`][Plottable] on your event
///   type and pass `&mut PlotObserver` as the solver observer. Works when the
///   event type is local to your crate.
/// - **Closure path** — Wrap `&mut PlotObserver` in a closure and call
///   [`record`][PlotObserver::record] manually. Use this for standard library
///   event types (e.g. `bisection::Event`) that carry lifetime parameters or
///   are foreign to your crate.
///
/// Call [`show`][PlotObserver::show] with a [`ShowConfig`] to render the result.
///
/// # Example — direct path
///
/// ```ignore
/// // Requires Plottable<2> impl on MyEvent (see Plottable docs).
/// let mut obs = PlotObserver::<2>::new(["Residual", "Step size"]);
/// my_solver::solve(&model, &problem, config, &mut obs)?;
/// obs.show(ShowConfig::new().title("My solver").legend())?;
/// ```
///
/// # Example — closure path
///
/// ```ignore
/// let mut obs = PlotObserver::<2>::new(["x", "Residual"]);
/// let mut iter = 0u32;
///
/// bisection::solve(&model, &problem, bracket, &config, |event: &bisection::Event<'_, _, _>| {
///     obs.record(
///         f64::from(iter),
///         [Some(event.x()), event.result().ok().map(|e| e.residuals[0])],
///     );
///     iter += 1;
///     None
/// })?;
///
/// obs.show(ShowConfig::new().title("Bisection").legend())?;
/// ```
pub struct PlotObserver<const N: usize> {
    names: [String; N],
    data: [Vec<[f64; 2]>; N],
}

impl<const N: usize> PlotObserver<N> {
    /// Creates a new `PlotObserver` with the given trace names.
    pub fn new(names: [&str; N]) -> Self {
        Self {
            names: names.map(str::to_owned),
            data: std::array::from_fn(|_| Vec::new()),
        }
    }

    /// Records a single data point across all traces.
    ///
    /// For each trace slot, `None` skips recording for that trace while
    /// leaving other traces unaffected.
    pub fn record(&mut self, x: f64, traces: [Option<f64>; N]) {
        for (i, y) in traces.into_iter().enumerate() {
            if let Some(y) = y {
                self.data[i].push([x, y]);
            }
        }
    }

    /// Opens a blocking egui window displaying all collected traces.
    ///
    /// Blocks until the window is closed by the user.
    ///
    /// # Errors
    ///
    /// Returns an error if the native window cannot be created.
    pub fn show(self, config: ShowConfig) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions::default();
        let title = config.title.unwrap_or_default();
        let traces: Vec<(String, Vec<[f64; 2]>)> = self.names.into_iter().zip(self.data).collect();

        eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| {
                Ok(Box::new(PlotApp {
                    traces,
                    legend: config.legend,
                    log_y: config.log_y,
                }))
            }),
        )
    }
}

impl<const N: usize, E, A> Observer<E, A> for PlotObserver<N>
where
    E: Plottable<N>,
{
    fn observe(&mut self, event: &E) -> Option<A> {
        if let Some(x) = event.x() {
            self.record(x, event.traces());
        }
        None
    }
}

/// Allows `&mut PlotObserver<N>` to be passed to solvers that take an observer
/// by value, so [`PlotObserver::show`] can be called after the solve completes.
impl<const N: usize, E, A> Observer<E, A> for &mut PlotObserver<N>
where
    E: Plottable<N>,
{
    fn observe(&mut self, event: &E) -> Option<A> {
        (*self).observe(event)
    }
}

/// The egui [`eframe::App`] that renders collected traces.
struct PlotApp {
    traces: Vec<(String, Vec<[f64; 2]>)>,
    legend: bool,
    log_y: bool,
}

impl eframe::App for PlotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut plot = Plot::new("plot_observer");
            if self.legend {
                plot = plot.legend(Legend::default());
            }
            if self.log_y {
                plot = plot.y_axis_label("log₁₀");
            }
            let log_y = self.log_y;
            plot.show(ui, |plot_ui| {
                for (name, points) in &self.traces {
                    let plot_points: PlotPoints = if log_y {
                        points
                            .iter()
                            .filter(|p| p[1] > 0.0)
                            .map(|p| [p[0], p[1].log10()])
                            .collect()
                    } else {
                        points.iter().copied().collect()
                    };
                    plot_ui.line(Line::new(plot_points).name(name));
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use twine_core::Observer;

    #[derive(Clone, Copy)]
    struct Event {
        x: Option<f64>,
        a: Option<f64>,
        b: Option<f64>,
    }

    impl Plottable<2> for Event {
        fn x(&self) -> Option<f64> {
            self.x
        }

        fn traces(&self) -> [Option<f64>; 2] {
            [self.a, self.b]
        }
    }

    fn make_observer() -> PlotObserver<2> {
        PlotObserver::new(["a", "b"])
    }

    fn points(obs: &PlotObserver<2>, trace: usize) -> &[[f64; 2]] {
        &obs.data[trace]
    }

    // Helper to call observe without needing to specify the action type at each call site.
    fn feed(obs: &mut PlotObserver<2>, event: Event) {
        let _: Option<()> = obs.observe(&event);
    }

    #[test]
    fn records_point_when_both_x_and_y_are_some() {
        let mut obs = make_observer();
        feed(
            &mut obs,
            Event {
                x: Some(1.0),
                a: Some(2.0),
                b: Some(3.0),
            },
        );
        assert_eq!(points(&obs, 0), [[1.0, 2.0]]);
        assert_eq!(points(&obs, 1), [[1.0, 3.0]]);
    }

    #[test]
    fn skips_all_traces_when_x_is_none() {
        let mut obs = make_observer();
        feed(
            &mut obs,
            Event {
                x: None,
                a: Some(1.0),
                b: Some(2.0),
            },
        );
        assert!(points(&obs, 0).is_empty());
        assert!(points(&obs, 1).is_empty());
    }

    #[test]
    fn skips_only_affected_trace_when_y_is_none() {
        let mut obs = make_observer();
        feed(
            &mut obs,
            Event {
                x: Some(1.0),
                a: None,
                b: Some(3.0),
            },
        );
        assert!(points(&obs, 0).is_empty());
        assert_eq!(points(&obs, 1), [[1.0, 3.0]]);
    }

    #[test]
    fn accumulates_points_across_multiple_events() {
        let mut obs = make_observer();
        feed(
            &mut obs,
            Event {
                x: Some(1.0),
                a: Some(10.0),
                b: Some(20.0),
            },
        );
        feed(
            &mut obs,
            Event {
                x: Some(2.0),
                a: Some(11.0),
                b: Some(21.0),
            },
        );
        feed(
            &mut obs,
            Event {
                x: Some(3.0),
                a: Some(12.0),
                b: Some(22.0),
            },
        );
        assert_eq!(points(&obs, 0), [[1.0, 10.0], [2.0, 11.0], [3.0, 12.0]]);
        assert_eq!(points(&obs, 1), [[1.0, 20.0], [2.0, 21.0], [3.0, 22.0]]);
    }

    #[test]
    fn never_returns_an_action() {
        let mut obs: PlotObserver<2> = PlotObserver::new(["a", "b"]);
        let action: Option<()> = obs.observe(&Event {
            x: Some(1.0),
            a: None,
            b: None,
        });
        assert!(action.is_none());
    }

    #[test]
    fn record_direct_call_stores_points() {
        let mut obs: PlotObserver<2> = PlotObserver::new(["a", "b"]);
        obs.record(1.0, [Some(10.0), None]);
        obs.record(2.0, [None, Some(20.0)]);
        assert_eq!(points(&obs, 0), [[1.0, 10.0]]);
        assert_eq!(points(&obs, 1), [[2.0, 20.0]]);
    }
}
