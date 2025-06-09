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
/// # Panics
///
/// Panics if the component's `call` method returns an error. This is currently
/// handled with `unwrap()` for simplicity.
///
/// # Example
/// ```ignore
/// use std::convert::Infallible;
/// use serde::{Serialize, Deserialize};
/// use twine_core::Component;
/// use twine_dev::run_component_server;
///
/// #[derive(Clone)]
/// struct MyComponent;
///
/// #[derive(Serialize, Deserialize, Clone, Default)]
/// struct MyInput { value: f64 }
///
/// #[derive(Serialize)]
/// struct MyOutput { result: f64 }
///
/// impl Component for MyComponent {
///     type Input = MyInput;
///     type Output = MyOutput;
///     type Error = Infallible;
///
///     fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
///         Ok(MyOutput { result: input.value * 2.0 })
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let my_component = MyComponent;
///     run_component_server(my_component).await;
/// }
/// ```
pub async fn run_component_server<F, C, I, O>(component_fn: F)
where
    F: Fn() -> C + Sync + Send + Clone + 'static,
    C: Component<Input = I, Output = O>,
    I: for<'de> Deserialize<'de> + Serialize + Send + Sync + Clone + Default + 'static,
    O: Serialize + Send + 'static,
{
    let calculate = warp::path("calculate")
        .and(warp::post())
        .and(warp::body::json())
        .map(move |input: I| {
            let component = component_fn();
            let output = component.call(input).unwrap();
            warp::reply::json(&output)
        });

    let schema = warp::path("schema")
        .and(warp::get())
        .map(|| warp::reply::json(&I::default()));

    let name = warp::path("name")
        .and(warp::get())
        .map(|| warp::reply::json(&std::any::type_name::<C>()));

    let static_dir = std::env::var("CARGO_MANIFEST_DIR").map_or_else(
        |_| PathBuf::from("static"),
        |dir| {
            let manifest_path = PathBuf::from(dir);
            // Try current package's static directory first
            let local_static = manifest_path.join("static");
            if local_static.exists() {
                local_static
            } else {
                // Fall back to twine-dev's static directory
                manifest_path
                    .parent()
                    .and_then(|workspace| workspace.join("twine-dev/static").canonicalize().ok())
                    .unwrap_or_else(|| PathBuf::from("static"))
            }
        },
    );

    let static_files = warp::fs::dir(static_dir);

    let routes = calculate.or(schema).or(name).or(static_files);

    println!("Server running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

