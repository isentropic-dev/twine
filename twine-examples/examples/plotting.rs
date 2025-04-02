use twine_plot::PlotApp;

fn main() {
    let app = PlotApp::new()
        .add_series(
            "first series",
            &[
                [0.0, 1.0],
                [1.0, 3.0],
                [2.0, 1.0],
                [3.0, 2.0],
                [4.0, 2.5],
                [5.0, 0.5],
            ],
        )
        .add_series(
            "second series",
            &[
                [0.0, 2.0],
                [1.0, 0.5],
                [2.0, 0.25],
                [3.0, 0.5],
                [4.0, 1.0],
                [5.0, 2.0],
            ],
        );

    app.run("Example Plot").unwrap();
}
