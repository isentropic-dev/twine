# Twine

Twine is a Rust framework for defining and solving numerical problems.

Models are useful for solving problems. Twine ties together your Model, a Problem you want to solve, and a Solver that does the work.

## How It Works

Define a model:

```rust
use std::convert::Infallible;
use twine_core::Model;

/// A simple polynomial: f(x) = x³ - 3x
struct Polynomial;

impl Model for Polynomial {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        Ok(x.powi(3) - 3.0 * x)
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
    type Error = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Self::Error> {
        Ok(x[0])
    }

    fn residuals(&self, _input: &f64, output: &f64) -> Result<[f64; 1], Self::Error> {
        Ok([output - self.0])
    }
}

let solution = bisection::solve_unobserved(
    &Polynomial, &Target(-2.0), [0.0, 2.0], &bisection::Config::default(),
).unwrap();

// solution.x = 1.0 (where x³ - 3x = -2)
```

Find the minimum or maximum by defining an optimization problem with the same model:

```rust
use std::convert::Infallible;
use twine_core::OptimizationProblem;

/// Define an objective from the model input and output.
/// Solvers choose whether to minimize or maximize.
struct ObjectiveOutput;

impl OptimizationProblem<1> for ObjectiveOutput {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Self::Error> {
        Ok(x[0])
    }

    fn objective(&self, _input: &f64, output: &f64) -> Result<f64, Self::Error> {
        Ok(*output)
    }
}

// Use with any optimization solver
// golden_section::minimize(&Polynomial, ObjectiveOutput, [-2.0, 2.0]) → x = 1.0
// golden_section::maximize(&Polynomial, ObjectiveOutput, [-2.0, 2.0]) → x = -1.0
// Same model, same problem, same bracket, just minimize vs maximize
```

These examples use a simple polynomial, but the same pattern works with any `Model`, including large, multi-physics engineering systems.

## Observers

Solvers are domain-agnostic and know nothing about what your model represents. Observers bridge that gap by receiving events during execution and steering solver behavior based on domain knowledge you provide.

```rust
use twine_core::Observer;
use twine_observers::{HasResidual, CanStopEarly};

/// Logs each iteration and stops early when the residual is good enough.
struct GoodEnough { tolerance: f64, min_iters: usize, iter: usize }

impl<E: HasResidual, A: CanStopEarly> Observer<E, A> for GoodEnough {
    fn observe(&mut self, event: &E) -> Option<A> {
        self.iter += 1;
        let r = event.residual();
        println!("iter {}: residual = {r:.6}", self.iter);

        if self.iter >= self.min_iters && r.abs() < self.tolerance {
            return Some(A::stop_early());
        }
        None
    }
}

let observer = GoodEnough { tolerance: 0.1, min_iters: 5, iter: 0 };
let solution = bisection::solve(
    &Polynomial, &Target(0.0), [0.0, 3.0], &bisection::Config::default(), observer,
).unwrap();

// iter 1: residual = 2.500000
// iter 2: residual = -2.750000
// iter 3: residual = -0.484375
// iter 4: residual = 0.785156
// iter 5: residual = 0.097656
// solution.status = StoppedByObserver
```

`GoodEnough` is not tied to bisection. It works with any solver whose events expose a residual and whose actions support early stopping. The real power shows up in domain-specific observers — for example, an observer that recognizes a thermodynamic constraint violation and tells the solver to search elsewhere, turning an unsolvable problem into a solvable one.

## Crates

- **`twine-core`**: The `Model` trait, Problem traits, and the `Observer` trait.
- **`twine-solvers`**: Solver algorithms organized by problem type (e.g., `equation::bisection`, `optimization::golden_section`).
- **`twine-observers`**: Capability traits for cross-solver observers (e.g., `HasResidual`, `CanStopEarly`) and visualization tools like `PlotObserver`.

## Twine Models

Twine is domain-agnostic by design. For opinionated, domain-specific models and model-building tools, see the companion project [Twine Models](https://github.com/isentropic-dev/twine-models).
