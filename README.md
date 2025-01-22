# Twine

**Note:** Twine has evolved beyond its original design. This README reflects its current direction, focusing on composability, a functional approach, and system abstraction.

## What is Twine?

Twine is an open-source Rust framework for functional and composable system modeling. Every component—whether a simple building block or a sophisticated higher-order system—is just a function. Because these functions are pure, they can be safely composed, making it easy to build and understand larger systems from simple, reusable parts. Twine provides the tools to support this composition while ensuring type safety and enforcing valid connections between components, making models more maintainable and reliable.

## A Functional Approach to System Modeling

Twine models systems as compositions of pure functions. Every component in Twine is a function that follows a strict contract: it receives a typed input, computes a typed output, and always produces the same output for the same input. This deterministic behavior ensures:

- **Testability:** Pure functions are easier to test in isolation since they have no hidden state or side effects.
- **Parallelism:** Independent components can be executed concurrently without race conditions, enabling efficient parallel computation.
- **Composability:** Systems can be built hierarchically, with composed components acting just like basic components.
- **Extensibility:** Higher-order components can encapsulate complex behavior without introducing side effects.

## Configurable Pure Components

Twine components are initialized with a factory function:

```rust
fn create(Config) -> Result<impl Fn(Input) -> Result<Output>>
```

This function takes a `Config`, which defines the component’s static parameters, and returns a function that transforms `Input` into `Output`. This approach ensures that once configured, the component remains purely functional and free of side effects. By performing initialization only once, the resulting function can focus on computing its outputs as efficiently as possible.

## Declarative Composition with `compose!`

Twine's `compose!` macro enables users to define higher-order components by declaratively wiring together individual components, specifying how each component's inputs are computed at runtime. This eliminates boilerplate and guarantees type correctness at compile time.

#### Example Usage

```rust
compose!(simulated_home, {
    // Define the inputs for this composed component.
    Input {
        simulation_time: f64,
        indoor: {
            occupancy: u32,
            temp_setpoint: f64,
        },
    }

    // A weather component that provides conditions for a given time.
    weather => hourly_weather {
        time: simulation_time,
    }

    // A building component that models the home.
    house => building {
        // Occupancy comes from the composed component's input.
        occupancy: indoor.occupancy,

        // Outdoor temperature comes from the weather component's output.
        outdoor_temp: weather.temperature,

        thermostat: building::Thermostat {
            setpoint: indoor.temp_setpoint,
        },
    }
});
```

Here, `simulated_home` itself becomes a higher-order component that can be used in further compositions just like a primitive component.

## Dependency Resolution & State Integration

Twine automatically detects and resolves dependency cycles in composed components using iterative solvers that converge on a consistent state. This ensures modular and predictable simulations within Twine’s purely functional modeling framework.

Additionally, Twine provides built-in numerical integration to evolve system states over time, ensuring seamless simulation of dynamic systems while maintaining a functionally pure interface.
