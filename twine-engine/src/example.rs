/// A component that adds some config-specified value to an input.
mod adder {
    pub(crate) struct Config {
        pub(crate) increment_by: i32,
        pub(crate) criteria: i32,
    }

    pub(crate) struct Input {
        pub(crate) value_in: i32,
    }

    #[derive(Debug)]
    pub(crate) struct Output {
        pub(crate) value_out: i32,
        pub(crate) is_small: bool,
    }

    pub(crate) fn create(config: &Config) -> impl Fn(&Input) -> Output {
        let Config {
            criteria,
            increment_by,
        } = *config;

        move |input| {
            let value_out = input.value_in + increment_by;
            let is_small = value_out < criteria;
            Output {
                value_out,
                is_small,
            }
        }
    }
}

/// A component that subtracts some config-specified value from an input.
mod subtractor {
    type Config = f64;
    type Input = f64;
    type Output = f64;

    pub(crate) fn create(config: Config) -> impl Fn(&Input) -> Output {
        let to_remove = config;
        move |input| input - to_remove
    }
}

/// A component that does some pure math.
mod mdas {
    pub(crate) struct Input {
        pub(crate) x: i32,
        pub(crate) y: i32,
    }

    #[derive(Debug, Default)]
    pub(crate) struct Output {
        pub(crate) mul: f64,
        pub(crate) div: f64,
        pub(crate) add: f64,
        pub(crate) sub: f64,
    }

    pub(crate) fn create() -> impl Fn(&Input) -> Output {
        call_me_maybe
    }

    fn call_me_maybe(input: &Input) -> Output {
        let Input { x, y } = input;
        let x = f64::from(*x);
        let y = f64::from(*y);
        Output {
            mul: x * y,
            div: x / y,
            add: x + y,
            sub: x - y,
        }
    }
}

// The DSL for a system component that uses the above components.
//
//  define_component! {
//      name: example
//      inputs:
//          start_from: i32
//      components:
//          adder_a: adder::create
//          adder_b: adder::create
//          another: adder::create
//          reducer: subtractor::create
//          mdas: mdas::create
//      connections:
//          inputs.start_from -> adder_a.value_in
//          adder_a.value_out -> adder_b.value_in
//          adder_b.value_out -> another.value_in
//          adder_a.value_out -> mdas.x
//          another.value_out -> mdas.y
//          mdas.mul -> reducer.input
//  }
//
// Below is the expanded code from the above macro:

struct ExampleConfig {
    pub(crate) adder_a: adder::Config,
    pub(crate) adder_b: adder::Config,
    pub(crate) another: adder::Config,
    pub(crate) reducer: f64,
}

struct ExampleInput {
    pub start_from: i32,
}

#[derive(Debug)]
struct ExampleOutput {
    pub adder_a: adder::Output,
    pub adder_b: adder::Output,
    pub another: adder::Output,
    pub reducer: f64,
    pub mdas: mdas::Output,
}

#[allow(clippy::similar_names)]
fn create_example(config: &ExampleConfig) -> impl Fn(&ExampleInput) -> ExampleOutput {
    // Initialize components from their factories.
    let adder_a = adder::create(&config.adder_a);
    let adder_b = adder::create(&config.adder_b);
    let another = adder::create(&config.another);
    let reducer = subtractor::create(config.reducer);
    let mdas = mdas::create();

    // Return a closure that calls each component in the right order.
    move |inputs: &ExampleInput| {
        let adder_a_output = adder_a(&adder::Input {
            value_in: inputs.start_from,
        });

        let adder_b_output = adder_b(&adder::Input {
            value_in: adder_a_output.value_out,
        });

        let another_output = another(&adder::Input {
            value_in: adder_b_output.value_out,
        });

        let mdas_output = mdas(&mdas::Input {
            x: adder_b_output.value_out,
            y: another_output.value_out,
        });

        let reducer_output = reducer(&mdas_output.mul);

        ExampleOutput {
            adder_a: adder_a_output,
            adder_b: adder_b_output,
            another: another_output,
            reducer: reducer_output,
            mdas: mdas_output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_system() {
        // Initialze a system component from its config.
        let config = ExampleConfig {
            adder_a: adder::Config {
                increment_by: 1,
                criteria: 10,
            },
            adder_b: adder::Config {
                increment_by: 5,
                criteria: 100,
            },
            another: adder::Config {
                increment_by: 10,
                criteria: 500,
            },
            reducer: 5.0,
        };
        let system = create_example(&config);

        // Call the system with different inputs.
        let input = ExampleInput { start_from: 0 };
        let output = system(&input);
        println!("After first call: {output:#?}");

        let input = ExampleInput { start_from: 50 };
        let output = system(&input);
        println!("After second call: {output:#?}");
    }
}
