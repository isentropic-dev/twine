# ARCHITECTURE

## Purpose

Twine is a functional, strongly typed Rust framework for building, composing, and executing system models.

This document defines Twine’s **stable architectural boundaries**: the core abstractions, the split between construction and execution, and the responsibility/dependency rules for crates and layers.

If a proposed feature does not clearly fit within these boundaries, it should be redesigned or placed elsewhere rather than bending the architecture.

---

## 1. Core Philosophy

Twine is built around one idea:

> **A model is a deterministic function from a typed input to a typed output, with explicit failure.**

From this follow the guiding principles:

- **Determinism**: for a given input (and captured parameters), the observable result is stable.
  - Internal caching is allowed if it does not change results.
  - Randomness, I/O, and time-varying data must be made explicit (e.g., provided through inputs or immutable construction-time parameters) so execution tooling can reason about evaluations.
- **Explicitness**: inputs, outputs, and failure modes are visible in types.
- **Correctness via types**: Twine encourages encoding invariants directly in model I/O types (e.g., constrained numeric values and unit-aware quantities) to shift checks from runtime to compile time.
- **Composability**: larger models can be built from smaller models.
- **Generic execution**: solvers and simulators implement algorithms; they do not encode domain knowledge.

Twine is code-first. Models are ordinary Rust types and functions, not a DSL or a runtime graph engine.

### Non-goals

Twine is not:

- A runtime graph engine or scheduler
- A monolithic simulation runtime
- A DSL or visual modeling tool
- A solver library with hard-coded domain assumptions

---

## 2. The Model Contract (Execution Boundary)

The central abstraction is `Model`:

- Maps a concrete `Input` type to a concrete `Output` type
- Has an explicit error surface
- Is side-effect free in observable behavior

This is Twine’s **execution boundary**:

- Everything **above** `Model` is about **construction** (how you build a model)
- Everything **below** `Model` is about **execution** (how you run it)

Associated artifacts (e.g., input/output snapshots) exist to support execution and inspection, but do not expand the `Model` contract.

Twine has no special runtime "system" abstraction. A "system" is simply any type that implements `Model`, whether written directly or assembled from other models/components.

---

## 3. Construction vs Execution

### Construction (user-owned, domain-owned)

Construction is how models are created:

- Wiring components together
- Adapting, combining, or wrapping models
- Assembling inputs and conventions

Construction is intentionally **user-driven** and **domain-specific**. Twine provides utilities, but does not require a single construction worldview.

### Execution (Twine-owned, algorithm-owned)

Execution is how models are run:

- Evaluating a model at a point
- Solving equations
- Running optimizations
- Advancing state through time

Execution is intentionally **generic** and **domain-agnostic**.

---

## 4. Problems and Solvers

Twine separates **problem semantics** (what the user means) from **numerical algorithms** (how to compute).

### Problems

A **Problem** is layered on top of a `Model`. It encodes semantics that solvers cannot infer, such as:

- How solver variables map to model inputs
- How model outputs are interpreted
- What constitutes residuals, objectives, constraints, or derivatives

Problem definitions do **not** describe physics, components, or wiring.

Problem families are extensible. Common examples include equation solving, optimization, and transient simulation; future problem types (e.g., bounded or constrained optimization) fit the same pattern.

#### Transient problems (time integration)

A transient problem defines:

- **State**: what is integrated
- **Derivatives**: how time derivatives are computed from model evaluations
- **Input reconstruction**: how an input is produced from state and time

**Continuous vs discrete contract:** to support higher-order and multi-stage integrators correctly:

- **Continuous evolution**: input reconstruction and derivative evaluation may occur multiple times per step (including intermediate stages).
- **Discrete control/application**: discrete updates (mode switches, thermostats, controller state updates) occur **once per accepted step**, after integration.

This separation prevents hidden discontinuities from invalidating integrator assumptions.

### Solvers

A **Solver** is an algorithm implemented as a function:

- Takes a `Model` and a `Problem`, plus any solver-specific inputs (e.g., bounds, initial guesses, or initial conditions)
- May accept configuration and observation hooks
- Runs to completion
- Returns a result (solution or failure)

Solvers own:

- Iteration/stepping strategy
- Convergence logic
- Error handling and recovery mechanisms
- Observation/event hooks

Solvers never encode domain assumptions.

---

## 5. Composition, Components, and Domain Foundations

Twine distinguishes **domain-agnostic composition mechanics** from **domain-specific modeling building blocks**.

### Composition utilities (domain-agnostic)

Composition utilities create new `Model`s from existing `Model`s. They are:

- Domain-agnostic
- Solver-agnostic
- Purely construction-time tools

They may use internal graphs or other data structures as construction artifacts, but runtime execution sees only a `Model`.

Domain-agnostic composition utilities must not encode physics or component-specific conventions. Such logic belongs in **Components** or **Domain Foundations**.

Example: a fluid-network builder (pumps/pipes/tanks) that enforces mass and energy conservation laws across component connections is domain logic, even if it produces a unified `Model`.

### Components (domain-specific)

Components are reusable domain building blocks (equipment models, controllers, kernels). They may:

- Implement `Model` directly, or
- Expose kernels that are wrapped into a `Model`

### Domain foundations (domain-specific, non-component)

Domain foundations are libraries that support components but are not themselves components:

- Property models and data (e.g., thermo properties)
- Domain types and reusable domain utilities

They are not solvers and not runtime execution frameworks.

Lifecycle guideline: **Domain Foundations** may begin life alongside **Components** while their API stabilizes. When reuse and dependency pressure warrant it, they should be extracted into dedicated crates. If a foundation becomes broadly useful outside Twine, it may be promoted to an external crate with its own versioning and lifecycle.

---

## 6. Inspection and Presentation

Twine separates:

- **Execution**: produces execution artifacts (e.g., model evaluations, solver iterations, solutions)
- **Inspection**: extracts and derives data from execution artifacts and selects what is of interest for downstream use
- **Presentation**: formats, stores, and visualizes inspected data

**Inspection** stands on its own. It enables:

- Online control and steering (early stopping, constraint detection, adaptive stepping)
- Logging and telemetry without a presentation framework
- Testing and validation (assertions, invariant checks, regression comparisons)

**Presentation** (plotting, storage, reports) is one consumer of inspected data, not the only one.

The *core* inspection surface is lightweight and domain-agnostic. Semantic enrichments are allowed as **opt-in** wrappers when useful (e.g., time bases for time-series, units metadata for plotting, named channels). These enrichments must not affect execution semantics.

---

## 7. Layers, Crates, and Dependency Rules

Twine is organized into conceptual layers.

**Dependency rule:** a layer may depend only on layers listed **above** it (more foundational), never on layers listed below it.

The list below is ordered by allowed dependency direction; it is not a runtime call stack. Layers are conceptual boundaries and may be implemented as one crate or many.

### Layers

- **Core**: minimal modeling contracts and invariant tools
- **Platform Foundations**: broadly reusable, non-domain foundations used across domains and by generic tooling (e.g., units/time)
- **Compose**: construction utilities and model-to-model adapters (construction-time)
- **Solve**: generic numerical algorithms (execution-time)
- **Inspect**: inspection and data extraction from executions
- **Presentation**: plotting, persistence, and reporting built on inspected data
- **Components**: domain-specific reusable models
- **Domain Foundations**: domain-specific property/data libraries

### Layer-specific invariants

The ordering above defines the general dependency direction. The rules below define **layer invariants** (what each layer is allowed to contain).

- **Core** (`twine-core`) must remain small and **ecosystem-agnostic**.
  - It may include dependency-light invariant tools that are broadly useful when defining model I/O (e.g., constrained numeric wrappers).
- **Platform Foundations** may depend on external ecosystems to provide shared, non-domain vocabulary and utilities without forcing those choices into **Core**.
  - Example: `twine-units` defines Twine’s physical-units/time vocabulary grounded in `uom`.
- **Solve** must remain **domain-agnostic**: solver APIs must not encode domain assumptions or require domain crates. Domain semantics belong in `Problem` definitions and domain layers.

### Tooling (build-time)

Build-time tooling is not part of the layer ordering above; any layer may use it.

- Proc-macro tooling (e.g., `twine-macros`) is optional ergonomics and must not introduce required runtime concepts.
- Tooling may generate construction code (including from connection descriptions/graphs), but the output participates in Twine through the same stable runtime contracts (e.g., `Model`).

### Representative crates (non-exhaustive examples)

- **`twine-core`** (Core): `Model`, `Snapshot`, and dependency-light invariant helpers for model I/O (e.g., constrained numeric types)
- **`twine-units`** (Platform Foundations): physical units and time vocabulary (grounded in `uom`, plus Twine-specific additions)
- **`twine-solve`** (Solve): equation/optimization/transient algorithms
- **`twine-components`** (Components): reusable domain components built on `Model`
- **`twine-thermo`** (Domain Foundations): thermodynamic property modeling and related domain utilities
- **`twine-inspect`** (Inspect): base inspection and extraction utilities
- **`twine-plot`** (Presentation): plotting, built on `twine-inspect`
- **`twine-store`** (Presentation): persistence and serialization, built on `twine-inspect`
- **`twine-macros`** (Tooling): optional proc-macros for ergonomic code generation
- **`twine-examples`**: end-to-end examples and validation cases

