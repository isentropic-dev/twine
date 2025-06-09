use std::{convert::Infallible, f64::consts::PI};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use twine_core::Component;
use uom::fmt::DisplayStyle;
use uom::si::area::square_meter;
use uom::si::{
    f64::{Area, Length},
    length::meter,
};

/// Component for calculating the area of a circle.
pub struct CircleArea;

/// Component for calculating the area of a rectangle.
pub struct RectangleArea;

/// Input structure for the `CircleArea` component.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CircleInput {
    /// The radius of the circle.
    #[serde(
        serialize_with = "serialize_length",
        deserialize_with = "deserialize_length"
    )]
    pub radius: Length,
}

impl Default for CircleInput {
    fn default() -> Self {
        CircleInput {
            radius: Length::new::<meter>(5.),
        }
    }
}

/// Input structure for the `RectangleArea` component.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RectangleInput {
    /// The length of the rectangle.
    #[serde(
        serialize_with = "serialize_length",
        deserialize_with = "deserialize_length"
    )]
    pub length: Length,

    /// The width of the rectangle.
    #[serde(
        serialize_with = "serialize_length",
        deserialize_with = "deserialize_length"
    )]
    pub width: Length,
}

impl Default for RectangleInput {
    fn default() -> Self {
        RectangleInput {
            length: Length::new::<meter>(5.),
            width: Length::new::<meter>(4.),
        }
    }
}

/// Output structure for `CircleArea` and `RectangleArea`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    /// The calculated area of the shape.
    #[serde(serialize_with = "serialize_area")]
    pub area: Area,
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

impl Component for RectangleArea {
    type Input = RectangleInput;
    type Output = Output;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(Self::Output {
            area: input.length * input.width,
        })
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_length<S>(length: &Length, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!(
        "{:?}",
        length.into_format_args(meter, DisplayStyle::Abbreviation)
    ))
}

fn deserialize_length<'de, D>(deserializer: D) -> Result<Length, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<Length>()
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse length: {e}")))
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_area<S>(area: &Area, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!(
        "{:?}",
        area.into_format_args(square_meter, DisplayStyle::Abbreviation)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        area::{square_centimeter, square_foot, square_mile},
        length::{foot, inch, kilometer},
    };

    #[test]
    fn circle_area_calculator() {
        let input = CircleInput {
            radius: Length::new::<kilometer>(1.0),
        };

        let output = CircleArea.call(input).unwrap();
        let square_miles = output.area.get::<square_mile>();

        assert_relative_eq!(square_miles, 1.212_976, epsilon = 1e-6);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let input = CircleInput {
            radius: Length::new::<meter>(5.0),
        };

        let json = serde_json::to_string(&input).unwrap();
        println!("Serialized: {json}");

        let deserialized: CircleInput = serde_json::from_str(&json).unwrap();
        assert!(
            (input.radius.get::<meter>() - deserialized.radius.get::<meter>()).abs() < f64::EPSILON
        );
    }

    #[test]
    fn rectangle_area_calculator() {
        let input = RectangleInput {
            length: Length::new::<inch>(3.0),
            width: Length::new::<foot>(1.0),
        };

        let output = RectangleArea.call(input).unwrap();
        let square_ft = output.area.get::<square_foot>();
        let square_cm = output.area.get::<square_centimeter>();

        assert_relative_eq!(square_ft, 0.25);
        assert_relative_eq!(square_cm, 232.2576);
    }
}
