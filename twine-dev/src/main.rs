use std::{convert::Infallible, f64::consts::PI, path::PathBuf};

use serde::{Deserialize, Serialize};
use twine_core::Component;
use warp::Filter;

#[derive(Clone)]
pub struct CircleArea;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CircleInput {
    /// The radius of the circle.
    pub radius: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    /// The calculated area of the shape.
    pub area: f64,
}

impl Component for CircleArea {
    type Input = CircleInput;
    type Output = Output;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(Self::Output {
            area: PI * input.radius * input.radius,
        })
    }
}

async fn run_component_server<C, I, O>(component: C) 
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
        .map(move || {
            warp::reply::json(&I::default())
        });
        
    let name = warp::path("name")
        .and(warp::get())
        .map(move || {
            warp::reply::json(&component_name)
        });

    let static_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(|dir| PathBuf::from(dir).join("static"))
        .unwrap_or_else(|_| PathBuf::from("static"));
    
    let static_files = warp::fs::dir(static_dir);

    let routes = calculate.or(schema).or(name).or(static_files);

    println!("Server running on http://localhost:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

#[tokio::main]
async fn main() {
    let circle_area = CircleArea;
    
    run_component_server(circle_area).await;
}
