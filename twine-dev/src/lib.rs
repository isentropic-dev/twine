use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use twine_core::Model;
use warp::Filter;

/// Runs a web server for any Component with separate DTO types for serialization.
///
/// The server provides:
/// - `/calculate` endpoint for POST requests with JSON input
/// - `/schema` endpoint for GET requests returning the default input schema
/// - `/name` endpoint for GET requests returning the component type name
/// - Static file serving from the `static/` directory
///
/// # Type Parameters
///
/// - `F`: Factory function that creates component instances
/// - `C`: Component type
/// - `I`: Component's actual input type (must implement Default)
/// - `O`: Component's actual output type
/// - `InputDto`: DTO type for deserializing JSON input
/// - `OutputDto`: DTO type for serializing JSON output
///
/// # Panics
///
/// Panics if the component's `call` method returns an error or if conversion fails.
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
/// #[derive(Default)]
/// struct MyInput { value: f64 }
/// struct MyOutput { result: f64 }
///
/// #[derive(Serialize, Deserialize, Clone)]
/// struct MyInputDto { value: f64 }
///
/// #[derive(Serialize)]
/// struct MyOutputDto { result: f64 }
///
/// impl From<MyInputDto> for MyInput {
///     fn from(dto: MyInputDto) -> Self {
///         MyInput { value: dto.value }
///     }
/// }
///
/// impl From<MyOutput> for MyOutputDto {
///     fn from(output: MyOutput) -> Self {
///         MyOutputDto { result: output.result }
///     }
/// }
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
///     ComponentServer::<MyInputDto, MyOutputDto>::run(|| MyComponent).await;
/// }
/// ```
/// A server builder for running components with specific DTO types.
pub struct ComponentServer<InputDto, OutputDto> {
    _phantom: std::marker::PhantomData<(InputDto, OutputDto)>,
}

impl<InputDto, OutputDto> ComponentServer<InputDto, OutputDto>
where
    InputDto: for<'de> Deserialize<'de> + Serialize + Send + Sync + Default + 'static,
    OutputDto: Serialize + Send + 'static,
{
    /// Run a component server with the specified DTO types.
    ///
    /// # Panics
    ///
    /// Panics if the component's `call` method returns an error or if conversion fails.
    ///
    /// # Example
    /// ```ignore
    /// ComponentServer::<MyInputDto, MyOutputDto>::run(|| MyComponent).await;
    /// ```
    pub async fn run<F, M, I, O>(component_fn: F)
    where
        F: Fn() -> M + Sync + Send + Clone + 'static,
        M: Model<Input = I, Output = O>,
        I: From<InputDto> + Send + 'static,
        O: Into<OutputDto> + Send + 'static,
    {
        let calculate = warp::path("calculate")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |input_dto: InputDto| {
                let component = component_fn();
                let input: I = input_dto.into();
                let output = component.call(input).unwrap();
                let output_dto: OutputDto = output.into();
                warp::reply::json(&output_dto)
            });

        let schema = warp::path("schema")
            .and(warp::get())
            .map(|| warp::reply::json(&InputDto::default()));

        let name = warp::path("name")
            .and(warp::get())
            .map(|| warp::reply::json(&std::any::type_name::<M>()));

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
                        .and_then(|workspace| {
                            workspace.join("twine-dev/static").canonicalize().ok()
                        })
                        .unwrap_or_else(|| PathBuf::from("static"))
                }
            },
        );

        let static_files = warp::fs::dir(static_dir);

        let routes = calculate.or(schema).or(name).or(static_files);

        println!("Server running on http://localhost:3030");
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    }

    /// Run a component server with the specified DTO types in a blocking manner.
    ///
    /// This creates its own tokio runtime internally, so you don't need tokio as a dependency.
    ///
    /// # Panics
    ///
    /// Panics if the component's `call` method returns an error or if conversion fails.
    ///
    /// # Example
    /// ```ignore
    /// ComponentServer::<MyInputDto, MyOutputDto>::run_blocking(|| MyComponent);
    /// ```
    pub fn run_blocking<F, M, I, O>(component_fn: F)
    where
        F: Fn() -> M + Sync + Send + Clone + 'static,
        M: Model<Input = I, Output = O>,
        I: From<InputDto> + Send + 'static,
        O: Into<OutputDto> + Send + 'static,
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(Self::run(component_fn));
    }
}
