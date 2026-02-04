# ARCHITECTURE

## Purpose

Twine is a functional, strongly typed Rust framework for building, composing, and executing system models.

This document defines Twine's **stable architectural boundaries**: the core abstractions and responsibilities that govern the framework's design.

If a proposed feature doesn't cleanly fit within these boundaries, it should be redesigned rather than bending the architecture.

---

## 1. Core Philosophy and Fundamental Distinctions

Twine is built around one idea:

> **A model is a deterministic, fallible function with a typed input, output, and error.**

From this follow the guiding principles:

- **Determinism**: For a given input (and captured parameters), the result is always the same.
  - Internal caching is allowed if it does not change results.
  - Randomness, I/O, and time-varying data must be made explicit (provided through inputs or immutable construction-time parameters).

- **Explicitness**: Inputs, outputs, and failure modes are visible in types.

- **Correctness via types**: Encode invariants directly in model I/O types (e.g., constrained numeric values, unit-aware quantities) to shift checks from runtime to compile time.

- **Composability**: Larger models can be built from smaller models.

- **Generic execution**: Solvers implement algorithms without encoding domain knowledge.

- **Code-first**: Models are ordinary Rust types and functions.

### Non-goals

Twine is not:

- A runtime graph engine or scheduler
- A monolithic simulation runtime
- A DSL or visual modeling tool
- A solver library with hard-coded domain assumptions

### Fundamental Distinctions

Twine's architecture is organized around two orthogonal distinctions:

#### Construction vs. Execution

- **Construction**: Building models (composition, wiring, domain implementation)
- **Execution**: Using models to solve problems (defining scenarios, extracting results)

The `Model` trait forms the boundary between these concerns.

#### Domain-Agnostic vs. Domain-Aware

- **Domain-Agnostic**: Generic algorithms and utilities that work across domains
- **Domain-Aware**: Specific domain knowledge (e.g., physical principles, user-defined model semantics)

These distinctions create a 2×2 matrix that defines Twine's architecture:

|                    | **Domain-Agnostic**              | **Domain-Aware**                 |
|--------------------|----------------------------------|----------------------------------|
| **Construction**   | Generic composition utilities    | Domain components and properties |
| **Execution**      | Solvers, observers, views        | Domain knowledge via extensions  |

Throughout this document, "domain knowledge" and "domain semantics" encompass two related concepts:

1. **General Domain Knowledge**: Understanding of domain-specific principles such as thermodynamics,
   fluid dynamics, electrical systems, or financial models.

2. **Model-Specific Knowledge**: User decisions about their specific model, such as which input
   parameters to vary, which output values are significant, or what criteria determine success.

Both forms of knowledge appear in two key places:
- Domain-Aware Construction: When building Models through Components and Domain Foundations
- Extension Points: When defining Problems, Observers, and Views that connect domain knowledge to execution

---

## 2. The Model Contract

The central abstraction is the `Model` trait:

- Maps a concrete `Input` type to a concrete `Output` type
- Defines an explicit error type
- Has no external side effects

This is Twine's **execution boundary**:

- Everything **above** `Model` is about **construction** (how you build a model)
- Everything **below** `Model` is about **execution** (how you run it)

Twine has no special runtime "system" abstraction. A "system" is simply any type that implements `Model`, whether written directly or assembled from other models/components.

---

## 3. Problems, Solvers, Observers, and Views

### Problems

A `Model` maps inputs to outputs but does not encode what question is being asked. Problems supply that context:

- Each solver family defines its own Problem trait
- Users implement these traits to provide domain-specific knowledge
- Example: `EquationProblem`, `OptimizationProblem`, `TransientProblem`

Problems are how domain knowledge connects to solvers.

### Solvers

A Solver is an algorithm implemented as a function. It takes a `Model` and a Problem (plus solver-specific inputs such as initial guesses or initial conditions), runs to completion, and returns a result.

Solvers own:

- Iteration and stepping strategy
- Convergence logic
- Error handling and recovery
- Observer event and action types

Solvers never encode domain assumptions.

### Observers

Observers allow domain-aware code to react to execution events and optionally steer solver behavior. The `Observer` trait is generic over Event and Action types: each solver family defines its own Event (what happened) and Action (what to do about it) types, while the trait interface remains uniform.

Observers enable:

- Online monitoring and logging
- Early stopping or steering based on domain criteria
- Incremental data capture for Views

Like Problems, Observers are an extension point: solvers define the Event and Action types that parameterize the trait, and users or components provide domain-aware implementations.

### Views

Views define what's meaningful in solutions and how to extract it:

- Each view family defines its own trait (e.g., `TimeSeriesView`, `ParametricView`)
- Users implement these traits to provide domain-specific knowledge
- Views extract domain-relevant data from execution artifacts

Views can be implemented in two ways:
1. Post-execution on complete Solution objects
2. During execution through Observers that capture data incrementally

This flexibility allows efficient handling of both small and large solution sets.

Downstream utilities such as plotting and persistence consume View output.

---

## 4. Architectural Invariants and Dependency Rules

### Domain-Agnostic → Domain-Aware

**Invariant**: Domain-agnostic code must never depend on domain-aware code.

- Generic composition utilities must not encode domain-specific knowledge
- Core solvers must remain domain-agnostic and work with any `Model`
- Generic view infrastructure must not depend on domain-specific semantics
- Platform Foundations should not contain domain-specific knowledge
- Example: `twine-compose` cannot depend on `twine-components`; `twine-solve` cannot depend on `twine-thermo`
- Example: a fluid-network builder (pumps/pipes/tanks) that enforces mass and energy conservation laws across component connections is domain logic, even if it produces a unified `Model`. Such logic belongs in Components or Domain Foundations, not in generic composition utilities.

### Construction → Execution

**Invariant**: Execution layers must depend only on the `Model` interface, not on how models are constructed.

- Solvers must accept any `Model` without requiring knowledge of how it was built
- Core execution mechanisms (solvers, view processing) remain generic
- Execution algorithms must not depend on component-specific implementation details
- Example: `twine-solve` depends on `twine-core` but not on the internals of `twine-components`

The reverse direction is allowed:

- Construction code may depend on execution APIs (e.g., Components may depend on solver traits to implement Problems or provide Observers)
- Problem, Observer, and View implementations are domain-aware code that depends on both execution traits and domain foundations
- These implementations live in user code or domain-aware crates, not in the generic execution crates

### Extension Points as Clean Interfaces

**Invariant**: Domain knowledge crosses boundaries only through well-defined extension points.

- **Problems**: Domain knowledge informs solvers what to solve
- **Observers**: Domain knowledge monitors and steers solver execution
- **Views**: Domain knowledge informs what to extract from solutions

### Component-Solver Relationship

**Invariant**: Components may use solvers internally, but this must be a one-way relationship.

- Components can use solvers during construction or to implement their `Model` interface
- Components provide domain-specific Problem and Observer implementations to guide solvers
- Solvers must never contain or depend on component-specific logic

### Core Access

All layers may depend on Core and Platform Foundations. Core contracts and platform utilities are universally available (e.g., all crates may depend on `twine-core` and `twine-units`).

---

## 5. Layer Organization

Twine's architecture follows this layered structure:

```
┌────────────────────────── FOUNDATION LAYERS ──────────────────────────┐
│                                                                       │
│  Core (Model trait)         Platform Foundations (e.g., units, time)  │
│                                                                       │
└───────────────────────────────────┬───────────────────────────────────┘
                                    │
                                    ▼
┌─────────── CONSTRUCTION ──────────┬─────────── EXECUTION ─────────────┐
│                                   │                                   │
│  DOMAIN-AGNOSTIC:                 │  DOMAIN-AGNOSTIC:                 │
│  • Compose                        │  • Solve (Problems, Observers)    │
│                                   │  • View (Views)                   │
│                                   │                                   │
│  DOMAIN-AWARE:                    │  DOMAIN-AWARE EXTENSIONS:         │
│  • Domain Foundations             │  • Problem implementations        │
│  • Components                     │  • Observer implementations       │
│                                   │  • View implementations           │
│                                   │                                   │
└───────────────────────────────────┴───────────────────────────────────┘
```

This structure maintains separation between:
1. Domain-agnostic and domain-aware code
2. Construction and execution concerns
3. Core foundations and specialized layers

---

## 6. Layers and Crates

This section describes the primary architectural layers of Twine. Each layer may span multiple crates. The crates mentioned are examples and not an exhaustive list.

### Core Foundations

- **Core** (`twine-core`): The central `Model` trait that defines the execution boundary
  - Contains the essential contracts and interfaces
  - Includes lightweight invariant tools for strongly typed interfaces

- **Platform Foundations** (`twine-units`): Broadly reusable, non-domain foundations
  - Provides shared vocabulary and utilities like units and time representations
  - Bridges to external ecosystems where appropriate (e.g., `uom` for units)

### Construction

- **Domain-Agnostic Construction**:
  - **Compose** (`twine-compose`): Generic composition patterns and model adapters
    - Provides tools for combining and transforming models
    - Supports construction of larger models from smaller ones

- **Domain-Aware Construction**:
  - **Domain Foundations** (`twine-thermo`): Domain-specific property models and utilities
    - Examples: thermodynamic properties, fluid properties, material data
    - Available to any domain-aware code, whether in construction (Components) or execution extensions (Problem/Observer implementations)
    - May evolve from component-adjacent code to dedicated crates, potentially becoming external dependencies

  - **Components** (`twine-components`): Reusable domain-specific building blocks
    - Built on Domain Foundations
    - May implement `Model` directly or provide building blocks for `Model` implementations

### Execution

- **Solve** (`twine-solve`): Generic numerical algorithms and Problem/Observer trait definitions
  - Implements solvers for different Problem types
  - Defines Problem and Observer traits as extension points

- **View** (`twine-view`): Generic data extraction and View definitions
  - Defines View traits as extension points
  - Downstream crates (e.g., `twine-plot`, `twine-persist`) consume View output for visualization and storage

### Tooling

Proc-macro crates (e.g., `twine-macros`) reduce boilerplate and improve ergonomics but must not introduce required runtime concepts. They may generate construction code (including from connection descriptions or graphs), but the output participates in Twine through the same stable contracts (e.g., `Model`).

Proc-macro crates sit outside the layer ordering; any layer may use them.
