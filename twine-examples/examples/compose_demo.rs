use twine_components::example::math::{Adder, Arithmetic, ArithmeticInput};
use twine_core::Component;
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

fn main() {
    run_math_example();
    run_other_example();
}

#[allow(clippy::float_cmp)]
fn run_math_example() {
    println!("\n====== MathExample ======");
    let input = Input { x: 10.0, y: 20.0 };
    println!("Input:");
    print_field("x", input.x);
    print_field("y", input.y);
    println!();

    let math = MathExample::new(MathComponents {
        add_one: Adder::new(1.0),
        add_two: Adder::new(2.0),
        math: Arithmetic,
    });

    let output = math.call(input).unwrap();

    assert_eq!(output.add_one, 11.0);
    assert_eq!(output.add_two, 22.0);
    assert_eq!(output.math.sum, 33.0);

    print_output(&output);
}

#[allow(clippy::float_cmp)]
fn run_other_example() {
    println!("\n====== OtherExample ======");
    let input = OtherInput { value: 5.0 };
    println!("Input:");
    print_field("value", input.value);
    println!();

    let other = OtherExample::new(MathComponents {
        add_one: Adder::new(1.0),
        add_two: Adder::new(2.0),
        math: Arithmetic,
    });

    let output = other.call(input).unwrap();

    assert_eq!(output.math.sum, 10.0);
    assert_eq!(output.math.product, 25.0);
    assert_eq!(output.add_one, 11.0);
    assert_eq!(output.add_two, 27.0);

    print_output(&output);
}

/// Prints labeled output fields for a composed math component.
fn print_output(output: &<() as MathComponentsTypes>::__Outputs) {
    println!("Output:");
    print_field("add_one", output.add_one);
    print_field("add_two", output.add_two);
    print_field("math.sum", output.math.sum);
    print_field("math.diff", output.math.difference);
    print_field("math.product", output.math.product);
    print_field("math.quotient", output.math.quotient);
    print_field("math.average", output.math.average);
}

/// Prints a single labeled value with consistent formatting.
fn print_field(label: &str, value: f64) {
    println!("  {label:<13} = {value}");
}
