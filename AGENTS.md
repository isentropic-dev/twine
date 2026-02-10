# Twine

Rust framework for numerical problem solving. See README.md for usage examples.

## Crates

- **twine-core** (`crates/core`): Shared traits and types
- **twine-solvers** (`crates/solvers`): Solver implementations, organized by problem type

## Core Abstraction

```
Model::call(input) -> output
Problem::input(x) -> model_input
Problem::residuals|objective(input, output) -> metric
Solver::solve(model, problem, bracket, config, observer) -> Solution
```

Solvers are generic over Model and Problem. Problems adapt solver variables (`x: [f64; N]`) 
to model inputs and extract metrics from outputs.

## Observer Pattern

Solvers emit events; observers optionally return actions.

```rust
trait Observer<Event, Action> {
    fn observe(&mut self, event: &Event) -> Option<Action>;
}

impl<E, A> Observer<E, A> for () {
    fn observe(&mut self, _: &E) -> Option<A> { None }
}
```

Events expose solver state (current point, bracket, errors).
Actions steer behavior (stop early, assume sign/worse for recovery).

## Solver Conventions

- Public functions: `solve`/`minimize`/`maximize` with observer, `*_unobserved` without
- Module structure: one file per concern (bracket, config, event, action, solution, error)
- Config validation at entry, not per-iteration
- `Solution` contains: status, solver variable, objective/residual, snapshot (input+output), iters

## Testing

Use `approx::assert_relative_eq!` for floating point comparisons.
