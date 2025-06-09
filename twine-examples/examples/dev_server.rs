//! Development server example showing how to create a web interface for the `RectangleArea` component.
//!
//! This example demonstrates:
//! - Using string-based DTOs for user-friendly input (e.g., "4 m", "5 m")
//! - Converting between DTOs and component types
//! - Running a component server with custom serialization
//!
//! The server provides a web interface where users can input rectangle dimensions
//! as strings with units and get the calculated area as a formatted string.
//!
//! Run with: `cargo run --example dev_server`
//! Then visit: <http://localhost:3030>

use serde::{Deserialize, Serialize};
use twine_components::example::area_calculators::{Output, RectangleArea, RectangleInput};
use twine_dev::ComponentServer;
use uom::{fmt::DisplayStyle, si::area::square_meter};

/// DTO for rectangle area input that accepts dimensions as strings with units.
///
/// This allows users to input values like "4 m" or "5 ft" in the web interface,
/// which are then parsed into the appropriate units for the component.
#[derive(Deserialize, Serialize)]
struct AreaInputDto {
    /// Length as a string with units (e.g., "4 m")
    length: String,
    /// Width as a string with units (e.g., "5 m")
    width: String,
}

impl Default for AreaInputDto {
    fn default() -> Self {
        Self {
            length: String::from("4 m"),
            width: String::from("5 m"),
        }
    }
}

impl From<AreaInputDto> for RectangleInput {
    fn from(value: AreaInputDto) -> Self {
        Self {
            length: value.length.parse().unwrap(),
            width: value.width.parse().unwrap(),
        }
    }
}

/// DTO for rectangle area output that formats the result as a string.
///
/// This converts the calculated area from the component's internal units
/// into a user-friendly string representation (e.g., "20 m²").
#[derive(Deserialize, Serialize)]
struct AreaOutputDto {
    /// Formatted area with units (e.g., "20 m²")
    area: String,
}

impl From<Output> for AreaOutputDto {
    fn from(value: Output) -> Self {
        Self {
            area: format!(
                "{:?}",
                value
                    .area
                    .into_format_args(square_meter, DisplayStyle::Abbreviation)
            ),
        }
    }
}

/// Runs a development server for the `RectangleArea` component.
///
/// The server will be available at <http://localhost:3030> and provides:
/// - A web interface for inputting rectangle dimensions
/// - JSON API endpoints for programmatic access
/// - Automatic conversion between user-friendly strings and component types
#[tokio::main]
async fn main() {
    ComponentServer::<AreaInputDto, AreaOutputDto>::run(|| RectangleArea).await;
}
