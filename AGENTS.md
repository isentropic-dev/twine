# Twine

Rust framework for numerical problem solving. See README.md for usage examples.

## Crates

- **twine-core** (`crates/core`): Shared traits and types
- **twine-solvers** (`crates/solvers`): Solver implementations, organized by problem type
- **twine-observers** (`crates/observers`): Reusable `Observer` implementations

## Core Abstraction

Three problem types: equation (root-finding), optimization, and ODE. All follow the same pattern:

- `Model::call(input) -> Result<output, error>`
- `Problem::input(x) -> model_input` — maps solver variables to model input
- `Problem` extracts a metric from input/output (residuals, objective, or derivative)

Solvers are generic over `Model` and `Problem`.

## Observer Pattern

`Observer<E, A>` is defined in `twine-core`. Closures and `()` both implement it — `()` is the no-op observer. Solvers emit events; observers optionally return actions to steer behavior (stop early, assume worse, etc.).

## Solver Conventions

- Public API: `solve`/`minimize`/`maximize` (with observer) and `*_unobserved` variants
- Config validation at entry, not per-iteration

## Testing

Use `approx::assert_relative_eq!` for floating point comparisons.
