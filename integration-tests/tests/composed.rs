#![allow(dead_code)]

use twine_components::example::math::{
    Adder, Arithmetic, ArithmeticInput, ArithmeticOutput, Multiplier,
};
use twine_core::{Component, Composable, Composed, Twine, TwineError};

/// Defines a generic structure for grouping math-related subcomponents.
///
/// `MathComponentsTemplate` serves as a template where each field represents a
/// subcomponent. It can be used in different contexts:
/// - To store actual subcomponent instances.
/// - To define the expected input types of subcomponents.
/// - To represent the computed output types of subcomponents.
struct MathComponentsTemplate<AddOne, DoubleIt, DoMath> {
    add_one: AddOne,
    double_it: DoubleIt,
    do_math: DoMath,
}

/// A type alias for a fully instantiated set of math components.
///
/// `MathComponents` binds `MathComponentsTemplate` to concrete subcomponent types.
type MathComponents = MathComponentsTemplate<Adder<f64>, Multiplier<f64>, Arithmetic>;

impl Composable for MathComponents {
    /// Defines the input types for each subcomponent.
    type Inputs = MathComponentsTemplate<
        <Adder<f64> as Component>::Input,
        <Multiplier<f64> as Component>::Input,
        <Arithmetic as Component>::Input,
    >;

    /// Defines the output types for each subcomponent.
    type Outputs = MathComponentsTemplate<
        <Adder<f64> as Component>::Output,
        <Multiplier<f64> as Component>::Output,
        <Arithmetic as Component>::Output,
    >;
}

/// Defines the input type for the composed math component.
struct MathInput {
    x: f64,
    y: f64,
}

/// A `Composed` implementation that chains `MathComponents` together.
///
/// This struct stores the result of `Twine::build()`, holding the composed
/// processing chain. It allows `MathExample` to function as a `Component` that
/// transforms `MathInput` into `<MathComponents as Composable>::Outputs`.
struct MathExample {
    component: Box<
        dyn Component<
            Input = MathInput,
            Output = <MathComponents as Composable>::Outputs,
            Error = TwineError,
        >,
    >,
}

impl Composed for MathExample {
    type Input = MathInput;
    type Components = MathComponents;
    type Error = TwineError;

    /// Builds a `Twine` chain that executes these operations in sequence:
    /// 1. Adds 1 to `input.x` (`add_one`).
    /// 2. Doubles the result (`double_it`).
    /// 3. Passes the doubled result and `input.y` to `Arithmetic` (`do_math`).
    /// 4. Collects the results into the `Outputs` variant of `MathComponentsTemplate`.
    fn new(components: Self::Components) -> Self {
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
                |(_input, add_one, double_it), do_math| (add_one, double_it, do_math),
            ))
            .then_fn(|(add_one, double_it, do_math)| MathComponentsTemplate {
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
        Output = <Self::Components as Composable>::Outputs,
        Error = TwineError,
    > {
        self.component.as_ref()
    }
}

#[test]
#[allow(clippy::float_cmp)]
fn composed_math_component_works() {
    let math = MathExample::new(MathComponents {
        add_one: Adder::new(1.0),
        double_it: Multiplier::new(2.0),
        do_math: Arithmetic,
    });

    let input = MathInput { x: 4.0, y: 2.0 };
    let output = math.call(input).unwrap();

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
        "Expected Arithmetic input with x = 10.0, y = 2.0"
    );
}
