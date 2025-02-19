#![allow(dead_code)]

use twine_components::example::math::{Adder, Arithmetic, ArithmeticInput};
use twine_core::compose;

#[compose]
struct Composed {
    add_one: Adder<f64>,
    add_two: Adder<f64>,
    first_math: Arithmetic,
    second_math: Arithmetic,
}

struct Input {
    x: f64,
    y: f64,
}

// This function demonstrates how component names like `add_one` can be used in
// different contexts, whether as their input type or output type. It enables
// full LSP support for renaming and "Go to Definition," making the code easier
// to navigate and modify.
//
// This approach allows users to define component connections using regular
// Rust code. Another macro can parse these connections to generate a dependency
// graph and determine the correct call order, ensuring each component has the
// necessary values available before being called.
fn connect(input: &Input, output: &ComposedOutputs) -> ComposedInputs {
    Composed {
        add_one: input.x,
        add_two: input.y,
        first_math: ArithmeticInput {
            x: output.add_one,
            y: output.add_two,
        },
        second_math: ArithmeticInput {
            x: output.first_math.sum,
            y: output.add_one,
        },
    }
}
