use twine_components::example::area_calculators::RectangleArea;
use twine_dev::run_component_server;

#[tokio::main]
async fn main() {
    let rect_area = RectangleArea;

    run_component_server(rect_area).await;
}
