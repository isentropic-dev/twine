use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoint};

/// A runnable egui application for plotting data.
#[derive(Default)]
pub struct PlotApp {
    series: Vec<Series>,
}

struct Series {
    name: String,
    points: Vec<PlotPoint>,
}

impl PlotApp {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn add_series(mut self, name: &str, points: &[[f64; 2]]) -> Self {
        self.series.push(Series {
            name: name.to_string(),
            points: points.iter().copied().map(Into::into).collect(),
        });

        self
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn run(self, name: &str) -> Result<(), eframe::Error> {
        eframe::run_native(
            name,
            eframe::NativeOptions::default(),
            Box::new(|_cc| Ok(Box::new(self))),
        )
    }
}

impl eframe::App for PlotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            Plot::new("plot-id")
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    for series in &self.series {
                        let points = series.points.as_slice();
                        let name = &series.name;

                        plot_ui.line(Line::new(points).name(name));
                    }
                });
        });
    }
}
