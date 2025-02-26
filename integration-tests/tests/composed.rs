#![allow(dead_code)]

use twine_components::example::math::{
    Adder, Arithmetic, ArithmeticInput, ArithmeticOutput, Multiplier,
};
use twine_core::{Component, ComponentGroup, Composed, Twine};

/// A marker type that defines the structure of the composed component.
///
/// Implementing `ComponentGroup` for `MathComposition` specifies:
/// - The number and names of subcomponents (e.g., `add_one`, `double_it`, `do_math`).
/// - The subcomponent types (`Adder<f64>`, `Multiplier<f64>`, `Arithmetic`).
/// - Their corresponding input and output types (`<Adder<f64> as Component>::Input`, etc.).
///
/// This type does not store component instances but only defines their structure.
/// The actual instances are stored in the generic `MathComponents`.
struct MathComposition;

/// Holds the actual subcomponent instances and their associated types.
///
/// `MathComponents` is a generic struct that defines and stores:
/// - The subcomponents themselves.
/// - Their input types.
/// - Their computed outputs.
///
/// This struct maintains consistent field names (`add_one`, `double_it`,
/// `do_math`), allowing Rust Analyzer to track references cleanly.
struct MathComponents<AddOne, DoubleIt, DoMath> {
    add_one: AddOne,
    double_it: DoubleIt,
    do_math: DoMath,
}

impl ComponentGroup for MathComposition {
    /// Defines the subcomponent types.
    type Components = MathComponents<Adder<f64>, Multiplier<f64>, Arithmetic>;

    /// Defines the input types for each subcomponent.
    type ComponentInputs = MathComponents<
        <Adder<f64> as Component>::Input,
        <Multiplier<f64> as Component>::Input,
        <Arithmetic as Component>::Input,
    >;

    /// Defines the output types for each subcomponent.
    type ComponentOutputs = MathComponents<
        <Adder<f64> as Component>::Output,
        <Multiplier<f64> as Component>::Output,
        <Arithmetic as Component>::Output,
    >;
}

/// A `Composed` implementation that chains `MathComponents` together.
///
/// Stores the `Twine`-built execution pipeline, allowing `Math` to function
/// as a `Component` that transforms `MathInput` into `<MathComposition as
/// ComponentGroup>::ComponentOutputs`.
struct Math {
    component: Box<
        dyn Component<
            Input = MathInput,
            Output = <MathComposition as ComponentGroup>::ComponentOutputs,
        >,
    >,
}

/// Defines the input type for the composed math component.
struct MathInput {
    x: f64,
    y: f64,
}

impl Composed for Math {
    type Input = MathInput;
    type Components = MathComposition;

    /// Builds a `Twine` chain that executes these operations in sequence:
    /// 1. Adds 1 to `input.x` (`add_one`).
    /// 2. Doubles the result (`double_it`).
    /// 3. Passes `input.y` and the doubled result to `Arithmetic` (`do_math`).
    /// 4. Bundles all results into `MathComponents` with labeled outputs.
    fn new(components: <Self::Components as ComponentGroup>::Components) -> Self {
        let component = Twine::<MathInput>::new()
            .then(components.add_one.map(
                |input: &MathInput| input.x,
                |input, add_one| (input, add_one),
            ))
            .then(components.double_it.map(
                |(_input, add_one): &(MathInput, f64)| *add_one,
                |(input, add_one), double_it| (input, add_one, double_it),
            ))
            .then(components.do_math.map(
                |(input, _add_one, double_it): &(MathInput, f64, f64)| ArithmeticInput {
                    x: *double_it,
                    y: input.y,
                },
                |(input, add_one, double_it), do_math| (input, add_one, double_it, do_math),
            ))
            .then_fn(|(_input, add_one, double_it, do_math)| MathComponents {
                add_one,
                double_it,
                do_math,
            })
            .build();

        Self {
            component: Box::new(component),
        }
    }

    /// Returns a reference to the composed processing chain.
    fn component(
        &self,
    ) -> &dyn Component<
        Input = Self::Input,
        Output = <Self::Components as ComponentGroup>::ComponentOutputs,
    > {
        self.component.as_ref()
    }
}

#[test]
#[allow(clippy::float_cmp)]
fn composed_math_component_works() {
    let math = Math::new(MathComponents {
        add_one: Adder::new(1.0),
        double_it: Multiplier::new(2.0),
        do_math: Arithmetic,
    });

    let input = MathInput { x: 4.0, y: 2.0 };
    let output = math.call(input);

    assert_eq!(output.add_one, 5.0, "Expected to add 1 to x");
    assert_eq!(
        output.double_it, 10.0,
        "Expected to double the previous result"
    );
    assert_eq!(
        output.do_math,
        ArithmeticOutput {
            sum: 12.0,
            difference: 8.0,
            product: 20.0,
            quotient: 5.0,
            average: 6.0,
        },
        "Arithmetic input should be (10, 2)"
    );
}
