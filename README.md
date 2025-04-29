# Twine

## What is Twine?

Twine is an open-source Rust framework for functional and composable system modeling.
Every component, whether a simple building block or a sophisticated higher-order system, implements a consistent interface.
By requiring components to be deterministic and always return the same output for the same input, Twine makes it easy to safely compose components, allowing users to build and understand larger systems from simple, reusable parts.
Twine provides the tools to support this composition while ensuring type safety and enforcing valid connections between components, making models more maintainable and reliable.

## A Functional Approach to System Modeling

Twine models systems as compositions of deterministic components.
Every component in Twine implements the `Component` trait with a `call` method that follows a strict contract: it receives a typed input, computes a typed output, and always produces the same output for the same input.
This behavior ensures:

- **Testability:** Deterministic components are easier to test in isolation since they have no hidden state or side effects.
- **Parallelism:** Independent components can be executed concurrently without race conditions, enabling efficient parallel computation.
- **Composability:** Systems can be built hierarchically, with composed components acting just like basic components.
- **Extensibility:** Higher-order components can encapsulate complex behavior without introducing side effects.

## Declarative Composition with Macros

Twine provides powerful macros that enable users to define higher-order components by declaratively wiring together individual components, specifying how each component's inputs are computed at runtime.
This approach eliminates boilerplate and guarantees type correctness at compile time.

```rust
#[composable]
pub struct BuildingComponents {
    // Provides hourly weather data including temperature, humidity, and solar radiation.
    pub weather: HourlyWeather,
    // Models thermal behavior of a building envelope with occupancy effects.
    pub building: Building,
    // Models heating, ventilation, and air conditioning system performance.
    pub hvac: HvacSystem,
}

/// Input parameters for the building simulation.
pub struct SimulationInput {
    /// Current simulation time.
    pub simulation_time: f64,
    /// Number of occupants in the building.
    pub occupancy: u32,
    /// Desired indoor temperature.
    pub temp_setpoint: f64,
}

#[compose(SimulatedHome)]
fn compose() {
    type Input = SimulationInput;
    type Components = BuildingComponents;

    Connections {
        weather: WeatherInput {
            time: input.simulation_time,
        },
        building: BuildingInput {
            // Occupancy from the composed component's input.
            occupancy: input.occupancy,
            // Current outdoor temperature from the weather component.
            outdoor_temp: output.weather.temperature,
            // The building receives heat input from the HVAC system.
            heat_input: output.hvac.heat_output,
        },
        hvac: HvacInput {
            // The HVAC system responds to the building's indoor temperature.
            indoor_temp: output.building.indoor_temp,
            outdoor_temp: output.weather.temperature,
            setpoint: input.temp_setpoint,
        },
    }
}
```

This generates a new `SimulatedHome` component that:
- Takes a simple `SimulationInput` with user-specified simulation parameters.
- Automatically routes values to the appropriate components.
- Resolves the feedback loop between building and HVAC components.
- Determines the correct execution order based on dependencies.

The generated component implements the `Component` trait, so it can be used just like any other component in further compositions.

## Dependency Resolution & State Integration

Twine automatically detects and resolves dependency cycles in composed components using iterative solvers that converge on a consistent state. It also identifies opportunities to execute independent components in parallel, improving performance without compromising modularity or reliability.

Additionally, Twine provides built-in numerical integration to evolve system states over time, ensuring seamless simulation of dynamic systems while maintaining a functionally pure interface.
