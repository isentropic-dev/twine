# Twine

Twine is a functional, strongly typed Rust framework for building, composing, and executing system models.

It is designed for engineers and researchers who want to write explicit, correct, and reusable modeling code, and to run that code using generic solvers, optimizers, and transient simulators.

Twine includes a growing set of domain components (thermal storage, heat exchangers, turbomachinery), thermodynamic property libraries, and shared execution tools that work with any model.

**Status**: Under active development. Core design is stable, but APIs may change between releases.

---

## Crates

| Crate | Description |
|-------|-------------|
| `twine-core` | `Model` trait, `Snapshot`, and invariant helpers (e.g., constrained numeric types) |
| `twine-units` | Physical units and time vocabulary grounded in `uom` |
| `twine-solve` | Equation solving, optimization, and transient integration |
| `twine-components` | Domain components (thermal storage, heat exchangers, turbomachinery, controllers) |
| `twine-thermo` | Thermodynamic property modeling |
| `twine-inspect` | Base inspection and extraction utilities |
| `twine-plot` | Plotting, built on `twine-inspect` |
| `twine-store` | Persistence and serialization, built on `twine-inspect` |
| `twine-macros` | Optional macros for common patterns |
| `twine-examples` | End-to-end examples |

---

## Getting Started

- Browse examples in [`twine-examples`](twine-examples/)
- Explore crate documentation: `cargo doc --open`
- Read the design philosophy and architectural boundaries in [`ARCHITECTURE.md`](ARCHITECTURE.md)
