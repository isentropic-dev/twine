use serde::{Deserialize, Serialize};
use twine_components::example::area_calculators::{Output, RectangleArea, RectangleInput};
use twine_dev::ComponentServer;
use uom::{fmt::DisplayStyle, si::area::square_meter};

#[derive(Deserialize, Serialize)]
struct AreaInputDto {
    length: String,
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

#[derive(Deserialize, Serialize)]
struct AreaOutputDto {
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

#[tokio::main]
async fn main() {
    ComponentServer::<AreaInputDto, AreaOutputDto>::run(|| RectangleArea).await;
}
