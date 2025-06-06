use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use twine_core::Component;
use warp::Filter;

/// Runs a web server for any Component that implements the required traits.
/// 
/// The server provides:
/// - `/calculate` endpoint for POST requests with JSON input
/// - `/schema` endpoint for GET requests returning the default input schema
/// - `/name` endpoint for GET requests returning the component type name
/// - Static file serving from the `static/` directory
/// 
/// # Example
/// ```no_run
/// use twine_dev::run_component_server;
/// 
/// #[tokio::main]
/// async fn main() {
///     let my_component = MyComponent;
///     run_component_server(my_component).await;
/// }
/// ```
pub async fn run_component_server<C, I, O>(component: C)
where
    C: Component<Input = I, Output = O> + Clone + Send + Sync + 'static,
    I: for<'de> Deserialize<'de> + Serialize + Send + Sync + Clone + Default + 'static,
    O: Serialize + Send + 'static,
{
    let calculate = warp::path("calculate")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |input: I| {
            let output = component.call(input).unwrap();
            warp::reply::json(&output)
        });

    let component_name = std::any::type_name::<C>();

    let schema = warp::path("schema")
        .and(warp::get())
        .map(move || warp::reply::json(&I::default()));

    let name = warp::path("name")
        .and(warp::get())
        .map(move || warp::reply::json(&component_name));

    let static_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(|dir| {
            let manifest_path = PathBuf::from(dir);
            // Try current package's static directory first
            let local_static = manifest_path.join("static");
            if local_static.exists() {
                local_static
            } else {
                // Fall back to twine-dev's static directory
                manifest_path.parent()
                    .and_then(|workspace| workspace.join("twine-dev/static").canonicalize().ok())
                    .unwrap_or_else(|| PathBuf::from("static"))
            }
        })
        .unwrap_or_else(|_| PathBuf::from("static"));

    let static_files = warp::fs::dir(static_dir);

    let routes = calculate.or(schema).or(name).or(static_files);

    println!("Server running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}