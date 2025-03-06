use twine_components::example::math::{Adder, Arithmetic, ArithmeticInput};
use twine_macros::{composable, compose};

/// The components to compose.
#[composable]
struct MathComponents {
    add_one: Adder<f64>,
    add_two: Adder<f64>,
    math: Arithmetic,
}

/// The input type for the composed component.
struct Input {
    x: f64,
    y: f64,
}

/// A different input type.
struct OtherInput {
    value: f64,
}

/// The composed math component.
#[compose(MathExample)]
fn compose() {
    type Input = Input;
    type Components = MathComponents;

    Connections {
        add_one: input.x,
        add_two: input.y,
        math: ArithmeticInput {
            x: output.add_one,
            y: output.add_two,
        },
    }
}

/// A different composed component using the same components.
#[compose(OtherExample)]
fn compose() {
    type Input = OtherInput;
    type Components = MathComponents;

    Connections {
        add_one: output.math.sum,
        add_two: output.math.product,
        math: ArithmeticInput {
            x: input.value,
            y: input.value,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use twine_core::Component;

    #[test]
    #[allow(clippy::float_cmp)]
    fn call_math_example() {
        let math = MathExample::new(MathComponents {
            add_one: Adder::new(1.0),
            add_two: Adder::new(2.0),
            math: Arithmetic,
        });

        let output = math.call(Input { x: 10., y: 20. }).unwrap();

        assert_eq!(output.add_one, 11.0);
        assert_eq!(output.add_two, 22.0);
        assert_eq!(output.math.sum, 33.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn call_other_example() {
        let other = OtherExample::new(MathComponents {
            add_one: Adder::new(1.0),
            add_two: Adder::new(2.0),
            math: Arithmetic,
        });

        let output = other.call(OtherInput { value: 5.0 }).unwrap();

        assert_eq!(output.math.sum, 10.0);
        assert_eq!(output.math.product, 25.0);
        assert_eq!(output.add_one, 11.0);
        assert_eq!(output.add_two, 27.0);
    }
}
