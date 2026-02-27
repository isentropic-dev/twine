//! Plotting observer for visualizing solver behavior.
//!
//! See [`PlotObserver`] for usage.

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
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
/// observer.show("Solver trace"); // blocks until the window is closed
/// ```
pub struct PlotObserver<E> {
    x_extractor: Extractor<E>,
    traces: Vec<Trace<E>>,
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
        }
    }

    /// Adds a named y-axis trace with its own extractor.
    ///
    /// Each call to `.trace()` registers one line on the plot. The closure
    /// receives the same event as the x-extractor; returning `None` skips the
    /// point for this trace while leaving other traces unaffected.
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
    /// This call blocks until the window is closed by the user.
    ///
    /// # Panics
    ///
    /// Panics if the native window cannot be created (e.g. no display available).
    pub fn show(self, title: &str) {
        let options = eframe::NativeOptions::default();
        let title = title.to_string();
        let traces: Vec<(String, Vec<[f64; 2]>)> = self
            .traces
            .into_iter()
            .map(|t| (t.name, t.points))
            .collect();

        eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| Ok(Box::new(PlotApp { traces }))),
        )
        .expect("failed to start egui window");
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

/// The egui [`eframe::App`] that renders the collected traces.
struct PlotApp {
    traces: Vec<(String, Vec<[f64; 2]>)>,
}

impl eframe::App for PlotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            Plot::new("plot_observer").show(ui, |plot_ui| {
                for (name, points) in &self.traces {
                    let plot_points = PlotPoints::new(points.clone());
                    plot_ui.line(Line::new(plot_points).name(name));
                }
            });
        });
    }
}
