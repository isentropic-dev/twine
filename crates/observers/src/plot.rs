//! Plotting observer for visualizing solver behavior.
//!
//! See [`PlotObserver`] for usage.

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use twine_core::Observer;

type Extractor<E> = Box<dyn Fn(&E) -> Option<f64>>;

/// An observer that collects trace data during solving and displays it via egui.
///
/// `PlotObserver` extracts x and y values from solver events using closures,
/// accumulates the data as the solver runs, and then renders an interactive
/// plot window when [`show`][PlotObserver::show] is called.
///
/// A point is only recorded when **both** the x and y extractors return
/// `Some(f64)`. Either extractor returning `None` causes that event to be
/// silently skipped for that trace.
///
/// The observer never returns an action; it is purely passive and does not
/// influence solver behavior.
///
/// # Example
///
/// ```ignore
/// use twine_observers::PlotObserver;
///
/// let mut observer = PlotObserver::new(|event: &MyEvent| Some(event.iteration as f64))
///     .trace("Residual", |event| Some(event.residual))
///     .trace("Step size", |event| event.step_size);
///
/// let solution = solver::solve(&model, &problem, bracket, &config, &mut observer);
///
/// observer.show("Solver trace")?; // blocks until the window is closed
/// ```
pub struct PlotObserver<E> {
    x_extractor: Extractor<E>,
    traces: Vec<Trace<E>>,
    legend: bool,
}

struct Trace<E> {
    name: String,
    y_extractor: Extractor<E>,
    points: Vec<[f64; 2]>,
}

impl<E> PlotObserver<E> {
    /// Creates a new `PlotObserver` with the given x-axis extractor.
    ///
    /// The closure is called for every solver event. If it returns `None` the
    /// event is skipped entirely (no trace point is recorded for any trace).
    pub fn new<X>(x_extractor: X) -> Self
    where
        X: Fn(&E) -> Option<f64> + 'static,
    {
        Self {
            x_extractor: Box::new(x_extractor),
            traces: Vec::new(),
            legend: false,
        }
    }

    /// Enables a legend that labels each trace by name.
    #[must_use]
    pub fn with_legend(mut self) -> Self {
        self.legend = true;
        self
    }

    /// Adds a named y-axis trace with its own extractor.
    ///
    /// Each call to `.trace()` registers one line on the plot. The closure
    /// receives the same event as the x-extractor; returning `None` skips the
    /// point for this trace while leaving other traces unaffected.
    #[must_use]
    pub fn trace<Y>(mut self, name: impl Into<String>, y_extractor: Y) -> Self
    where
        Y: Fn(&E) -> Option<f64> + 'static,
    {
        self.traces.push(Trace {
            name: name.into(),
            y_extractor: Box::new(y_extractor),
            points: Vec::new(),
        });
        self
    }

    /// Opens a blocking egui window that displays all collected traces.
    ///
    /// Blocks until the window is closed by the user.
    ///
    /// # Errors
    ///
    /// Returns an error if the native window cannot be created.
    pub fn show(self, title: &str) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions::default();
        let title = title.to_string();
        let traces: Vec<(String, Vec<[f64; 2]>)> = self
            .traces
            .into_iter()
            .map(|t| (t.name, t.points))
            .collect();

        let legend = self.legend;
        eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| Ok(Box::new(PlotApp { traces, legend }))),
        )
    }
}

impl<E, A> Observer<E, A> for PlotObserver<E> {
    fn observe(&mut self, event: &E) -> Option<A> {
        if let Some(x) = (self.x_extractor)(event) {
            for trace in &mut self.traces {
                if let Some(y) = (trace.y_extractor)(event) {
                    trace.points.push([x, y]);
                }
            }
        }
        None
    }
}

/// Allows `&mut PlotObserver<E>` to be passed to solvers that take an observer
/// by value, so [`PlotObserver::show`] can be called after the solve completes.
impl<E, A> Observer<E, A> for &mut PlotObserver<E> {
    fn observe(&mut self, event: &E) -> Option<A> {
        (*self).observe(event)
    }
}

/// Displays pre-collected trace data in an interactive plot window.
///
/// Useful when you cannot pass a [`PlotObserver`] directly to a solver —
/// for example, when solver events carry lifetime parameters that prevent
/// satisfying the solver's `for<'a> Observer<Event<'a, ...>, ...>` bound.
/// In that case, collect data manually in a closure, then call this function.
///
/// Opens a blocking egui window. Returns an error if the window cannot be
/// created. Blocks until the user closes the window.
///
/// # Example
///
/// ```ignore
/// use twine_observers::show_traces;
///
/// let mut points: Vec<[f64; 2]> = Vec::new();
/// solver::solve(&model, &problem, bracket, &config, |event| {
///     points.push([event.x(), event.residual()]);
///     None
/// })?;
///
/// show_traces("My solver", vec![("Residual".into(), points)])?;
/// ```
///
/// # Errors
///
/// Returns an error if the native window cannot be created.
pub fn show_traces(
    title: &str,
    traces: Vec<(String, Vec<[f64; 2]>)>,
    legend: bool,
) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    let title = title.to_string();
    eframe::run_native(
        &title,
        options,
        Box::new(move |_cc| Ok(Box::new(PlotApp { traces, legend }))),
    )
}

/// The egui [`eframe::App`] that renders the collected traces.
struct PlotApp {
    traces: Vec<(String, Vec<[f64; 2]>)>,
    legend: bool,
}

impl eframe::App for PlotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut plot = Plot::new("plot_observer");
            if self.legend {
                plot = plot.legend(Legend::default());
            }
            plot.show(ui, |plot_ui| {
                for (name, points) in &self.traces {
                    let plot_points: PlotPoints = points.iter().copied().collect();
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

    fn observer() -> PlotObserver<Event> {
        PlotObserver::new(|e: &Event| e.x)
            .trace("a", |e| e.a)
            .trace("b", |e| e.b)
    }

    fn points(obs: &PlotObserver<Event>, trace: usize) -> &[[f64; 2]] {
        &obs.traces[trace].points
    }

    // Helper to call observe without needing to specify the action type at each call site.
    fn feed(obs: &mut PlotObserver<Event>, event: Event) {
        let _: Option<()> = obs.observe(&event);
    }

    #[test]
    fn records_point_when_both_x_and_y_are_some() {
        let mut obs = observer();
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
        let mut obs = observer();
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
        let mut obs = observer();
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
        let mut obs = observer();
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
        let mut obs: PlotObserver<Event> = PlotObserver::new(|e: &Event| e.x);
        let action: Option<()> = obs.observe(&Event {
            x: Some(1.0),
            a: None,
            b: None,
        });
        assert!(action.is_none());
    }
}
