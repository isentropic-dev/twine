# Twine: A Composable Engine for Transient System Modeling

**Note:** This README describes the proposed design and approach for Twine. The details, including the component plugin API and system architecture, are subject to change as development progresses. The initial repository is simply a Rust library scaffold, and this document outlines the vision for the project.

## What is Twine?

Twine is an open-source Rust library designed to model transient systems using a composable, component-based approach. It aims to provide a powerful, flexible engine for engineers and developers, enabling efficient and modular simulation of dynamic systems.

## What isn't Twine?

Twine focuses purely on the simulation engine. It does not handle user interfaces, data visualization, or input/output management. These responsibilities are left to the users, who can integrate Twine into their broader workflows.

## Goals

- Model transient systems using connected components where each component operates as a pure function.
- Ensure extensibility through a plugin-like system for adding custom components.
- Separate numerical integration concerns (the "engine") from the system's structural representation (the "system").

---

## How it Works

### Components

Components are the building blocks of a Twine system. Each component is conceptually a pure function: given the same inputs, it always produces the same outputs.

#### Inputs and Outputs

- **Regular Inputs:** Immutable values representing external factors at a specific moment (e.g., flow rates, ambient temperature). The component reads these values but does not influence them.

- **State Inputs:** Immutable values representing the component's state at a specific moment. For each state input, the component must provide its time derivative, describing how the value changes over time.

- **Regular Outputs:** Values derived from the current inputs, representing results or computed properties at the same moment in time. These outputs do not influence the evolution of state inputs.

- **State Derivatives:** The time derivatives of the state inputs, provided by the component to predict how the system evolves.

### System

A system represents the graph of connected components, tracking how outputs feed into inputs. The system ensures all components are in a consistent state at each simulation step through an iterative convergence process.

### Engine

The engine advances the system forward in time using numerical integration. It supports various methods (e.g., Euler, Runge-Kutta) that are swappable via traits, allowing users to choose or implement integration strategies suitable for their needs.

---

## Plugin System

Twine supports dynamically loaded components (plugins) written in any language compatible with the C ABI. Plugins follow a standardized lifecycle and implement specific functions.

### Required Functions (C)

Plugins must export the following functions with the specified C ABI, implementing their equivalent behavior in the chosen programming language:

```c
// A structure to describe the component's metadata.
typedef struct {
    uint32_t num_inputs;    // Number of regular inputs.
    uint32_t num_states;    // Number of state inputs.
    uint32_t num_outputs;   // Number of regular outputs.
} ComponentMetadata;

// Initialize the component using a configuration string.
void* create(const char* config);

// Provide metadata about the component.
ComponentMetadata describe(const void* instance);

// Evaluate the component's outputs and state derivatives.
void evaluate(
    const void* instance,
    const double* inputs,
    const double* states,
    double* outputs,
    double* state_derivatives
);

// Clean up and free memory associated with the component.
void destroy(void* instance);
```

### Rust Component Trait

For Rust plugins, the `Component` trait must be implemented. It provides equivalent functionality:

```rust
/// A struct to describe the component's metadata.
pub struct ComponentMetadata {
    /// Number of regular inputs.
    pub num_inputs: usize,
    /// Number of state inputs.
    pub num_states: usize,
    /// Number of regular outputs.
    pub num_outputs: usize,
}

/// A trait representing a Twine component, with lifecycle methods for initialization, evaluation, and cleanup.
pub trait Component {
    /// Creates a new instance of the component using the provided configuration string.
    ///
    /// # Parameters
    /// - `config`: A string containing the configuration for the component.
    ///
    /// # Returns
    /// A raw pointer to the newly created component instance.
    fn create(config: &str) -> *mut Self;

    /// Describes the component's metadata, including the number of inputs, states, and outputs.
    ///
    /// # Returns
    /// A `ComponentMetadata` struct containing the component's metadata.
    fn describe(&self) -> ComponentMetadata;

    /// Evaluates the component's outputs and state derivatives based on the current inputs and states.
    ///
    /// # Parameters
    /// - `inputs`: A slice of the current regular inputs.
    /// - `states`: A slice of the current state inputs.
    /// - `outputs`: A mutable slice for storing the computed regular outputs.
    /// - `state_derivatives`: A mutable slice for storing the computed state derivatives.
    fn evaluate(
        &self,
        inputs: &[f64],
        states: &[f64],
        outputs: &mut [f64],
        state_derivatives: &mut [f64],
    );

    /// Cleans up and destroys the component instance, freeing any associated resources.
    ///
    /// # Parameters
    /// - `instance`: A raw pointer to the component instance to be destroyed.
    fn destroy(instance: *mut Self);
}
```
