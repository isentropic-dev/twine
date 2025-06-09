use std::{convert::Infallible, f64::consts::PI};

use twine_core::Component;
use uom::si::f64::{Area, Length};

/// Component for calculating the area of a circle.
pub struct CircleArea;

/// Component for calculating the area of a rectangle.
pub struct RectangleArea;

/// Input structure for the `CircleArea` component.
#[derive(Debug)]
pub struct CircleInput {
    /// The radius of the circle.
    pub radius: Length,
}

/// Input structure for the `RectangleArea` component.
#[derive(Debug)]
pub struct RectangleInput {
    /// The length of the rectangle.
    pub length: Length,

    /// The width of the rectangle.
    pub width: Length,
}

/// Output structure for `CircleArea` and `RectangleArea`.
#[derive(Debug)]
pub struct Output {
    /// The calculated area of the shape.
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
