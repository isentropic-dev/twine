# Twine

**Note:** Twine has evolved beyond its original design. This README reflects its current direction, focusing on composability, a functional approach, and system abstraction.

## What is Twine?

Twine is an open-source Rust framework for functional and composable system modeling. In Twine, every component, from the simplest building block to the most complex higher-order system, is just another function. It provides a declarative, functionally pure modeling framework where all components, whether simple or higher-order, are pure functions that transform structured inputs into outputs.

## A Functional Approach to System Modeling

Twine provides a structured framework where systems are modeled as compositions of pure functions. Every initialized component in Twine adheres to a strict contract: it receives a typed input and returns a typed output. This approach ensures:

- **Composability:** Systems can be built hierarchically, with composed components acting just like basic components.
- **Predictability:** Given the same input, a component will always produce the same output.
- **Extensibility:** Higher-order components can encapsulate complex behavior without introducing side effects.

## Configurable Pure Components

Every Twine component follows a structured configuration process. Each component provides a factory function:

```rust
fn create(config: Config) -> Result<impl Fn(Input) -> Result<Output>>;
```

This function takes a `Config`, which defines the component’s static parameters, and returns a pure function that transforms `Input` into `Output`. This ensures that once configured, the component remains purely functional and free of side effects. By handling initialization once, the resulting function can focus on computing its outputs as efficiently as possible.

## Declarative Composition with `compose!`

Twine's `compose!` macro enables users to define higher-order components by declaratively wiring together individual components. This eliminates boilerplate and guarantees type correctness at compile time.

#### Example Usage

```rust
compose!(simulated_home, {
    Input {
        time: f64,
        indoor: {
            occupancy: u32,
            temp_setpoint: f64,
        },
    }

    weather => hourly_weather {
        time,
    }

    house => building {
        occupancy: indoor.occupancy,
        outdoor_temp: weather.temperature,
        thermostat: building::Thermostat {
            setpoint: indoor.temp_setpoint,
        },
    }
});
```

Here, `simulated_home` itself becomes a higher-order component that can be used in further compositions just like a primitive component.

## Dependency Resolution & State Integration

Twine automatically detects and resolves dependency cycles within composed components by wrapping them into iterative solvers that converge on a consistent state. Twine embraces a purely functional framework for system modeling, enabling users to construct modular and predictable simulations.

Additionally, Twine provides built-in numerical integration to evolve system states over time, ensuring seamless simulation of dynamic systems while maintaining a functionally pure interface.

Every component—whether atomic or a complex system—is just a function, making composition, analysis, and reasoning straightforward and scalable.
