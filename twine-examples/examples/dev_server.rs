use twine_components::example::area_calculators::RectangleArea;
use twine_dev::run_component_server;

#[tokio::main]
async fn main() {
    run_component_server(|| RectangleArea).await;
}
