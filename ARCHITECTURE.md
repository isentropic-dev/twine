# ARCHITECTURE

## Purpose

Twine is a functional, strongly typed Rust framework for composing models and using them to solve problems.

This document defines Twine's **stable architectural boundaries**: the abstractions, responsibilities, and dependency rules that shape the framework.

If a proposed feature doesn't fit cleanly within these boundaries, the feature should be redesigned rather than bending the architecture.

---

## 1. Philosophy

Twine is built on a simple premise:

> **A model is a deterministic, fallible function with a typed input, output, and error.**

This premise, together with the following principles, guides every design decision:

- **Explicitness**: Inputs, outputs, and failure modes are visible in types.

- **Type-driven correctness**: Encode invariants in types to catch errors at compile time, not runtime.

- **Determinism**: For a given input, a model always produces the same result.

- **Composability**: Larger models can be built from smaller models.

- **Domain-agnostic solving**: Solvers implement algorithms without encoding domain knowledge.

- **Code-first**: Models are ordinary Rust types and functions.

The architecture is organized around two orthogonal distinctions:

- **Construction vs. Execution**: Construction is composing models. Execution is using models to solve problems. The `Model` trait forms the boundary between these concerns.

- **Domain-Agnostic vs. Domain-Aware**: Domain-agnostic code works across domains without encoding domain knowledge. Domain-aware code encodes specific domain knowledge — physical principles, model-specific decisions, or user-defined semantics.

Together, these distinctions organize the architecture:

|                    | **Domain-Agnostic**                | **Domain-Aware**                     |
|--------------------|-------------------------------------|--------------------------------------|
| **Construction**   | Composition utilities and adapters  | Components and domain-specific libraries |
| **Execution**      | Solver algorithms and trait definitions | Problem, Observer, and View implementations |

---

## 2. The Model Contract

The `Model` trait is the central abstraction and the boundary between construction and execution:

- Maps a concrete `Input` type to a concrete `Output` type
- Defines an explicit error type
- Must be deterministic: for a given input and fixed configuration, the result is always the same

Internal caching is allowed if it does not change results. Sources of nondeterminism (e.g., randomness) must be explicit inputs or fixed at construction time.

Any type that implements `Model` gains access to all of Twine's execution infrastructure, whether it was written directly or assembled from other models.

---

## 3. Execution

Execution has four concepts: **Problems** define what question to ask, **Solvers** answer it, **Observers** monitor and steer the process, and **Views** extract meaning from the solution.

### Problems

A `Model` maps inputs to outputs but does not encode what question is being asked. A Problem supplies that context. Each solver family defines its own Problem trait (e.g., `EquationProblem`, `OptimizationProblem`, `TransientProblem`), and users implement these traits to connect domain knowledge to solvers.

### Solvers

A Solver is a domain-agnostic algorithm. It takes a `Model` and a Problem (plus solver-specific inputs such as bounds or initial conditions), runs to completion, and returns a solution. Solvers own iteration strategy, convergence logic, error handling, and the Observer event and action types they define.

### Observers

Observers allow domain-aware code to monitor and steer solver execution. Each solver defines its own Event (what happened) and Action (what to do about it) types. Users implement the `Observer` trait to react to these events for logging, early stopping, or error recovery.

### Views

Views extract domain-relevant data from solutions. Users implement View traits to define what is meaningful in a solution. Views can operate post-execution on a complete solution or during execution through Observers that capture data incrementally. Downstream utilities such as plotting consume View output.

---

## 4. Invariants and Dependency Rules

### Domain-Agnostic Must Not Depend on Domain-Aware

Domain-agnostic code must never depend on domain-aware code. Domain knowledge enters the framework only through extension points: Problem, Observer, and View implementations.

For example, a thermodynamic cycle builder that enforces mass and energy conservation across component connections is domain logic, even if it produces a unified `Model`. Such logic belongs in domain-aware crates, not in generic composition utilities.

### Execution Must Not Depend on Construction

Execution code must depend only on the `Model` trait, not on how models are constructed.

The reverse is allowed: construction code may depend on execution APIs. Components may use solvers internally, and may provide Problem and Observer implementations to guide them. But solvers must never depend on component-specific logic.

### Foundation Access

All layers may depend on domain-agnostic foundations (the `Model` trait, shared utilities like constraints, physical units, and time).

---

## 5. Layers

The preceding concepts map to this layered structure:

```
┌──────────────────────────── FOUNDATIONS ───────────────────────────────┐
│                                                                       │
│  Model trait, constraints, physical units, time                       │
│                                                                       │
└───────────────────────────────────┬───────────────────────────────────┘
                                    │
                                    ▼
┌─────────── CONSTRUCTION ──────────┬─────────── EXECUTION ─────────────┐
│                                   │                                   │
│  DOMAIN-AGNOSTIC:                 │  DOMAIN-AGNOSTIC:                 │
│  • Compose                        │  • Solve                          │
│                                   │  • View                           │
│                                   │                                   │
│  DOMAIN-AWARE:                    │  DOMAIN-AWARE:                    │
│  • Domain libraries               │  • Problem implementations        │
│  • Components                     │  • Observer implementations       │
│                                   │  • View implementations           │
│                                   │                                   │
└───────────────────────────────────┴───────────────────────────────────┘
```

---

## 6. Crates

Each layer from the diagram above maps to one or more crates.

### Foundations

- `twine-core`: The `Model` trait, supporting types, and constraint primitives
- `twine-units`: Physical units and time representations, bridging to external ecosystems (e.g., `uom`)

### Construction

- `twine-compose`: Generic composition utilities and model adapters
- `twine-thermo`: Thermodynamic and fluid property models, an example of a domain-specific library
- `twine-components`: Reusable domain-specific building blocks, including `Model` implementations and utilities for constructing them

### Execution

- `twine-solve`: Solver algorithms and the Problem and Observer trait definitions
- `twine-view`: View trait definitions and generic data extraction
- `twine-plot`: Visualization of View output

### Tooling

- `twine-macros`: Proc-macro crate for reducing boilerplate

Macros are convenience, not magic. They reduce boilerplate and improve ergonomics, but must not introduce concepts that a user couldn't implement by hand. Any layer may use them.
