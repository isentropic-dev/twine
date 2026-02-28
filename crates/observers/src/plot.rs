//! Plotting observer for visualizing solver behavior.
//!
//! See [`PlotObserver`] and [`ShowConfig`] for usage.

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoint, PlotPoints, Text};

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
    /// y values are transformed with log₁₀ before plotting.
    /// Non-positive values are silently skipped.
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

/// An observer that collects trace data during solving and displays it via egui.
///
/// The const generic `N` is the number of traces. Create with
/// [`PlotObserver::new`], passing the trace names. Record data by capturing
/// `&mut PlotObserver` in a closure passed to the solver, calling
/// [`record`][PlotObserver::record] on each event.
///
/// Call [`show`][PlotObserver::show] with a [`ShowConfig`] to render the result.
///
/// # Example
///
/// ```ignore
/// let mut obs = PlotObserver::<2>::new(["x", "Residual"]);
/// let mut iter = 0_u32;
///
/// bisection::solve(&model, &problem, bracket, &config, |event: &bisection::Event<'_, _, _>| {
///     let residual = event.result().ok().map(|e| e.residuals[0].abs());
///     obs.record(f64::from(iter), [Some(event.x()), residual]);
///     iter += 1;
///     None
/// })?;
///
/// obs.show(ShowConfig::new().title("Bisection").legend().log_y())?;
/// ```
pub struct PlotObserver<const N: usize> {
    names: [String; N],
    data: [Vec<[f64; 2]>; N],
    labels: Vec<(f64, f64, String)>,
    label_size: f32,
}

impl<const N: usize> PlotObserver<N> {
    /// Creates a new `PlotObserver` with the given trace names.
    pub fn new(names: [&str; N]) -> Self {
        Self {
            names: names.map(str::to_owned),
            data: std::array::from_fn(|_| Vec::new()),
            labels: Vec::new(),
            label_size: 14.0,
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

    /// Places a text label at an arbitrary plot coordinate.
    ///
    /// Labels are rendered on top of all traces when [`show`][PlotObserver::show]
    /// is called. Font size is controlled by [`label_size`][PlotObserver::label_size].
    pub fn label(&mut self, x: f64, y: f64, text: impl Into<String>) {
        self.labels.push((x, y, text.into()));
    }

    /// Sets the font size for all text labels. Default is `14.0`.
    pub fn label_size(&mut self, size: f32) -> &mut Self {
        self.label_size = size;
        self
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
                    labels: self.labels,
                    label_size: self.label_size,
                    legend: config.legend,
                    log_y: config.log_y,
                    plot_rect: None,
                }))
            }),
        )
    }
}

/// Which axis gutter the cursor is hovering over.
#[derive(Clone, Copy)]
enum Gutter {
    /// Left gutter — zooms the y-axis.
    Y,
    /// Bottom gutter — zooms the x-axis.
    X,
}

/// The egui [`eframe::App`] that renders collected traces.
struct PlotApp {
    traces: Vec<(String, Vec<[f64; 2]>)>,
    labels: Vec<(f64, f64, String)>,
    label_size: f32,
    legend: bool,
    log_y: bool,
    /// Inner plot rect from the previous frame, used for gutter hit-testing.
    plot_rect: Option<egui::Rect>,
}

impl eframe::App for PlotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Determine which gutter (if any) the cursor is over, using the
            // inner plot rect captured from the previous frame.
            let gutter = self.plot_rect.and_then(|plot_rect| {
                let cursor = ctx.input(|i| i.pointer.latest_pos())?;
                if cursor.x < plot_rect.left()
                    && (plot_rect.top()..=plot_rect.bottom()).contains(&cursor.y)
                {
                    Some(Gutter::Y)
                } else if cursor.y > plot_rect.bottom()
                    && (plot_rect.left()..=plot_rect.right()).contains(&cursor.x)
                {
                    Some(Gutter::X)
                } else {
                    None
                }
            });

            // Read scroll delta only when needed — consuming it from the
            // input state signals intent to the rest of egui.
            let scroll_delta = if gutter.is_some() {
                ctx.input(|i| i.smooth_scroll_delta)
            } else {
                egui::Vec2::ZERO
            };

            // Compute the zoom factor to apply this frame, if any.
            // Guard on non-zero delta to avoid disabling auto-bounds spuriously.
            let zoom = gutter.and_then(|g| {
                if scroll_delta.y == 0.0 {
                    return None;
                }
                let f = (scroll_delta.y / 200.0).exp();
                Some(match g {
                    Gutter::Y => egui::Vec2::new(1.0, f),
                    Gutter::X => egui::Vec2::new(f, 1.0),
                })
            });

            let mut plot = Plot::new("plot_observer");
            if self.legend {
                plot = plot.legend(Legend::default());
            }
            if self.log_y {
                plot = plot.y_axis_label("log₁₀");
            }
            // When the cursor is over a gutter, disable scroll-to-pan so the
            // plot doesn't consume the scroll events we're using for zoom.
            if gutter.is_some() {
                plot = plot.allow_scroll(false);
            }

            let log_y = self.log_y;
            let label_size = self.label_size;
            let response = plot.show(ui, |plot_ui| {
                if let Some(factor) = zoom {
                    plot_ui.zoom_bounds_around_hovered(factor);
                }
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
                for (x, y, text) in &self.labels {
                    plot_ui.text(
                        Text::new(
                            PlotPoint::new(*x, *y),
                            egui::RichText::new(text).size(label_size),
                        )
                        .anchor(egui::Align2::LEFT_BOTTOM),
                    );
                }
                // Capture the inner plot rect for gutter hit-testing next frame.
                *plot_ui.transform().frame()
            });

            self.plot_rect = Some(response.inner);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_observer() -> PlotObserver<2> {
        PlotObserver::new(["a", "b"])
    }

    fn points(obs: &PlotObserver<2>, trace: usize) -> &[[f64; 2]] {
        &obs.data[trace]
    }

    #[test]
    fn records_point_when_both_traces_are_some() {
        let mut obs = make_observer();
        obs.record(1.0, [Some(2.0), Some(3.0)]);
        assert_eq!(points(&obs, 0), [[1.0, 2.0]]);
        assert_eq!(points(&obs, 1), [[1.0, 3.0]]);
    }

    #[test]
    fn skips_only_affected_trace_when_y_is_none() {
        let mut obs = make_observer();
        obs.record(1.0, [None, Some(3.0)]);
        assert!(points(&obs, 0).is_empty());
        assert_eq!(points(&obs, 1), [[1.0, 3.0]]);
    }

    #[test]
    fn accumulates_points_across_multiple_calls() {
        let mut obs = make_observer();
        obs.record(1.0, [Some(10.0), Some(20.0)]);
        obs.record(2.0, [Some(11.0), Some(21.0)]);
        obs.record(3.0, [Some(12.0), Some(22.0)]);
        assert_eq!(points(&obs, 0), [[1.0, 10.0], [2.0, 11.0], [3.0, 12.0]]);
        assert_eq!(points(&obs, 1), [[1.0, 20.0], [2.0, 21.0], [3.0, 22.0]]);
    }

    #[test]
    fn independent_x_values_per_trace() {
        let mut obs = make_observer();
        obs.record(1.0, [Some(10.0), None]);
        obs.record(2.0, [None, Some(20.0)]);
        assert_eq!(points(&obs, 0), [[1.0, 10.0]]);
        assert_eq!(points(&obs, 1), [[2.0, 20.0]]);
    }

    #[test]
    fn accumulates_labels_across_multiple_calls() {
        let mut obs = make_observer();
        obs.label(1.0, 2.0, "a");
        obs.label(3.0, 4.0, "b");
        assert_eq!(
            obs.labels,
            vec![(1.0, 2.0, "a".to_owned()), (3.0, 4.0, "b".to_owned())]
        );
    }
}
