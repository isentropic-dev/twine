# Twine

Twine is a Rust framework for defining and solving numerical problems.

A numerical model describes behavior, but on its own it only maps inputs to outputs. To solve an equation, optimize a design, or simulate a system, you need to ask the right question. Twine makes models useful by providing Problems to frame the question and Solvers to answer it.

You bring the model. Twine brings the machinery.

## How It Works

Define a model:

```rust
use std::convert::Infallible;
use twine_core::Model;

/// A simple polynomial: f(x) = x² - 4
struct Polynomial;

impl Model for Polynomial {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        Ok(x * x - 4.0)
    }
}
```

Find where the output equals a target by defining an equation problem:

```rust
use std::convert::Infallible;
use twine_core::EquationProblem;
use twine_solvers::equation::bisection;

/// Drive the model output to a target value.
struct Target(f64);

impl EquationProblem<1> for Target {
    type Input = f64;
    type Output = f64;
    type InputError = Infallible;
    type ResidualError = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Self::InputError> {
        Ok(x[0])
    }

    fn residuals(&self, _input: &f64, output: &f64) -> Result<[f64; 1], Self::ResidualError> {
        Ok([output - self.0])
    }
}

let solution = bisection::solve_unobserved(
    &Polynomial, &Target(0.0), [0.0, 5.0], &bisection::Config::default(),
).unwrap();

// solution.x ≈ 2.0 (a root of x² - 4)
```

Find the minimum by defining an optimization problem with the same model:

```rust
use std::convert::Infallible;
use twine_core::{OptimizationProblem, Minimize};
use twine_solvers::optimization::golden_section;

/// Minimize the model output.
struct Minimum;

impl OptimizationProblem<1> for Minimum {
    type Goal = Minimize;
    type Input = f64;
    type Output = f64;
    type InputError = Infallible;
    type ObjectiveError = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Self::InputError> {
        Ok(x[0])
    }

    fn objective(&self, _input: &f64, output: &f64) -> Result<f64, Self::ObjectiveError> {
        Ok(*output)
    }
}

let solution = golden_section::solve_unobserved(
    &Polynomial, &Minimum, [-5.0, 5.0], &golden_section::Config::default(),
).unwrap();

// solution.x ≈ 0.0 (same model, different question)
```

The model doesn't change. The question does. These examples use a simple polynomial, but the same pattern works with any `Model`, including large, multi-physics engineering systems.

## Observers

Solvers are domain-agnostic and have no knowledge of what your model represents. Observers bridge that gap. They receive events from the solver during execution and can steer its behavior based on domain knowledge you provide.

For example, an Observer might stop a solve early when a physical constraint is violated or help a solver recover from a model evaluation failure. The solver provides the algorithm; the Observer provides the intelligence.

## Crates

- **`twine-core`**: The `Model` trait, Problem traits, and the `Observer` trait. Zero dependencies.
- **`twine-solvers`**: Solver algorithms organized by problem type (e.g., `equation::bisection`, `optimization::golden_section`).
- **`twine-observers`**: Ready-to-use `Observer` implementations for plotting, logging, and persistence.

## Twine Components

Twine is domain-agnostic by design. It knows nothing about physics, units, or any specific engineering domain.

[Twine Components](https://github.com/isentropic-dev/twine-components) is a companion project that provides domain-aware building blocks for engineering models: thermodynamic properties, reusable components, and unit-aware types. It depends on Twine but Twine never depends on it.

## Status

Under active development. Core design is stable, but APIs may change between releases.
