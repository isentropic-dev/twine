# Golden Section Solver — Design Spec

## Legacy Code Notice

The current implementations of `search.rs`, `state.rs`, and `driver.rs` are legacy code. Do not attempt to preserve patterns, reconcile approaches, or refactor incrementally. Overwrite completely per this spec.

## Architecture

Three pieces:

1. **`evaluate()`** — Use existing function from `optimization/evaluate.rs`
2. **`State`** — Bracket + two interior Points + best (Point + Snapshot)
3. **`search()`** — Loop logic + observer interaction inline (no Driver)

## Types

### Point (existing, in `event.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub objective: f64,
}
```

For AssumeWorse recovery: `objective = transform(f64::INFINITY)`.

This works because both transforms are involutions:
- minimize: `transform = identity`, `transform(INFINITY) = INFINITY`
- maximize: `transform = negation`, `transform(INFINITY) = -INFINITY`

When computing score for comparison: `score = transform(objective)` yields `INFINITY` (worst) in both cases.

### Event (existing, in `event.rs`)

Keep current structure with `Evaluated`, `ModelFailed`, `ProblemFailed` variants.

### Config (modified)

Private fields, validated at construction:

```rust
pub struct Config {
    max_iters: usize,
    x_abs_tol: f64,
    x_rel_tol: f64,
}

impl Config {
    pub fn new(max_iters: usize, x_abs_tol: f64, x_rel_tol: f64) -> Result<Self, ConfigError>;
    pub fn max_iters(&self) -> usize;
    pub fn x_abs_tol(&self) -> f64;
    pub fn x_rel_tol(&self) -> f64;
}

impl Default for Config {
    fn default() -> Self {
        // Known-good values, unwrap is safe
        Self::new(100, 1e-12, 1e-12).unwrap()
    }
}
```

Entry points (`minimize`, `maximize`) assume valid Config—no validation call needed.

### ShrinkDirection (new, in `state.rs`)

```rust
enum ShrinkDirection {
    ShrinkLeft(f64),   // shrink left bound; payload is x for new inner_right
    ShrinkRight(f64),  // shrink right bound; payload is x for new inner_left
}
```

### State (rewritten)

```rust
pub(super) struct State<I, O> {
    bracket: GoldenBracket,
    left: Point,
    right: Point,
    best_point: Point,
    best_snapshot: Snapshot<I, O>,
}

impl<I, O> State<I, O> {
    pub(super) fn new(
        bracket: GoldenBracket,
        left: Point,
        right: Point,
        best_point: Point,
        best_snapshot: Snapshot<I, O>,
    ) -> Self;

    // Getters for event context
    pub(super) fn left(&self) -> Point;
    pub(super) fn right(&self) -> Point;
    pub(super) fn best_point(&self) -> Point;

    /// Pure query: which direction to shrink and where to evaluate next.
    pub(super) fn next_action<F: Fn(f64) -> f64>(&self, transform: &F) -> ShrinkDirection;

    /// Apply shrink and update interior point with new evaluation.
    pub(super) fn apply(&mut self, direction: ShrinkDirection, point: Point);

    /// Update best if this point has better score. Only call with real evaluations (not AssumeWorse).
    pub(super) fn maybe_update_best<F: Fn(f64) -> f64>(
        &mut self,
        point: &Point,
        transform: &F,
        snapshot: Snapshot<I, O>,
    );

    pub(super) fn is_converged(&self, config: &Config) -> bool;

    pub(super) fn into_solution(self, status: Status, iters: usize) -> Solution<I, O>;
}
```

No `pending` field. State tracks current bracket and points; `search()` handles control flow.

## GoldenBracket (add query methods)

Add methods that compute the new interior point position WITHOUT mutating the bracket:

```rust
impl GoldenBracket {
    /// Returns x for new inner_left after shrinking right (without mutating).
    pub(super) fn new_inner_left(&self) -> f64 {
        let new_right = self.inner_right;
        let new_width = new_right - self.left;
        self.left + (1.0 - INV_PHI) * new_width
    }

    /// Returns x for new inner_right after shrinking left (without mutating).
    pub(super) fn new_inner_right(&self) -> f64 {
        let new_left = self.inner_left;
        let new_width = self.right - new_left;
        new_left + INV_PHI * new_width
    }
}
```

## Invariant

At least one interior point always has a finite objective:

1. Init guarantees this (both infinite → error after init completes)
2. Each iteration shrinks away from the worse point (higher score), replacing it
3. The kept point is always the better one (finite beats infinite)

## Init

Evaluate both interior points, then decide based on outcomes:

```rust
let left_result = evaluate(model, problem, [bracket.inner_left]);
let right_result = evaluate(model, problem, [bracket.inner_right]);

match (left_result, right_result) {
    (Ok(left_eval), Ok(right_eval)) => {
        // Emit Evaluated event for right (other=left_point, best=left_point)
        // Observer can: None (continue), StopEarly, AssumeWorse
        // Build state with both points
    }
    (Ok(left_eval), Err(right_err)) => {
        // Emit ModelFailed or ProblemFailed for right (other=left_point, best=left_point)
        // Observer can: StopEarly, AssumeWorse, or None (propagate error)
    }
    (Err(left_err), Ok(right_eval)) => {
        // Emit ModelFailed or ProblemFailed for left (other=right_point, best=right_point)
        // Observer can: StopEarly, AssumeWorse, or None (propagate error)
    }
    (Err(_left_err), Err(right_err)) => {
        // No event emitted
        // Return Err using right_err (map to Error::Model or Error::Problem)
    }
}
```

After building state, if both points have infinite objective → logic error (should not happen with correct AssumeWorse handling, but check defensively).

### Init event emission detail

For `(Ok, Ok)`:
```rust
let left_point = Point { x: left_eval.x[0], objective: left_eval.objective };
let right_point = Point { x: right_eval.x[0], objective: right_eval.objective };

let event = Event::Evaluated {
    point: right_point,
    input: &right_eval.snapshot.input,
    output: &right_eval.snapshot.output,
    other: left_point,
    best: left_point,  // left evaluated first, so it's "best so far"
};

match observer.observe(&event) {
    Some(Action::StopEarly) => return Ok(Solution { status: StoppedByObserver, ... }),
    Some(Action::AssumeWorse) => {
        // right_point objective becomes transform(INFINITY)
        // right_snapshot is None (not eligible for best)
    }
    None => {
        // Use actual right_point and right_snapshot
    }
}
```

For `(Ok, Err)` — emit failure event for right:
```rust
let event = match &right_err {
    EvalError::Model(e) => Event::ModelFailed { x: bracket.inner_right, input: ..., other: left_point, best: left_point, error: e },
    EvalError::Problem(e) => Event::ProblemFailed { x: bracket.inner_right, input: ..., output: ..., other: left_point, best: left_point, error: e },
};

match observer.observe(&event) {
    Some(Action::StopEarly) => return Ok(Solution { status: StoppedByObserver, ... }),
    Some(Action::AssumeWorse) => {
        // Build state with right as Point { x, objective: transform(INFINITY) }
    }
    None => return Err(map_eval_error(right_err)),
}
```

For `(Err, Ok)` — emit failure event for left:
```rust
// Similar pattern, but other=right_point, best=right_point
```

## Main Loop

```rust
for iter in 1..=config.max_iters() {
    if state.is_converged(config) {
        return Ok(state.into_solution(Status::Converged, iter - 1));
    }

    let direction = state.next_action(&transform);
    let eval_x = match direction {
        ShrinkDirection::ShrinkLeft(x) | ShrinkDirection::ShrinkRight(x) => x,
    };

    // Context for events: the point not being replaced, and current best
    let other = match direction {
        ShrinkDirection::ShrinkLeft(_) => state.left(),   // left stays, right replaced
        ShrinkDirection::ShrinkRight(_) => state.right(), // right stays, left replaced
    };
    let best = state.best_point();

    let result = evaluate(model, problem, [eval_x]);

    let (point, snapshot) = match result {
        Ok(eval) => {
            let point = Point { x: eval.x[0], objective: eval.objective };
            let event = Event::Evaluated {
                point,
                input: &eval.snapshot.input,
                output: &eval.snapshot.output,
                other,
                best,
            };
            match observer.observe(&event) {
                Some(Action::StopEarly) => {
                    return Ok(state.into_solution(Status::StoppedByObserver, iter));
                }
                Some(Action::AssumeWorse) => {
                    let worse_point = Point { x: eval_x, objective: transform(f64::INFINITY) };
                    (worse_point, None)
                }
                None => (point, Some(eval.snapshot)),
            }
        }
        Err(err) => {
            let event = match &err {
                EvalError::Model(e) => Event::ModelFailed { x: eval_x, ..., other, best, error: e },
                EvalError::Problem(e) => Event::ProblemFailed { x: eval_x, ..., other, best, error: e },
            };
            match observer.observe(&event) {
                Some(Action::StopEarly) => {
                    return Ok(state.into_solution(Status::StoppedByObserver, iter));
                }
                Some(Action::AssumeWorse) => {
                    let worse_point = Point { x: eval_x, objective: transform(f64::INFINITY) };
                    (worse_point, None)
                }
                None => return Err(map_eval_error(err)),
            }
        }
    };

    state.apply(direction, point);
    if let Some(snap) = snapshot {
        state.maybe_update_best(&point, &transform, snap);
    }
}

Ok(state.into_solution(Status::MaxIters, config.max_iters()))
```

## State.next_action

```rust
fn next_action<F: Fn(f64) -> f64>(&self, transform: &F) -> ShrinkDirection {
    let left_score = transform(self.left.objective);
    let right_score = transform(self.right.objective);

    if left_score <= right_score {
        // Left is better → shrink right
        ShrinkDirection::ShrinkRight(self.bracket.new_inner_left())
    } else {
        // Right is better → shrink left
        ShrinkDirection::ShrinkLeft(self.bracket.new_inner_right())
    }
}
```

## State.apply

```rust
fn apply(&mut self, direction: ShrinkDirection, point: Point) {
    match direction {
        ShrinkDirection::ShrinkRight(_) => {
            // Shrinking right: [left, inner_right] becomes new bracket
            // Old inner_left becomes new inner_right
            // New point becomes new inner_left
            self.bracket.shrink_right();
            self.right = self.left;
            self.left = point;
        }
        ShrinkDirection::ShrinkLeft(_) => {
            // Shrinking left: [inner_left, right] becomes new bracket
            // Old inner_right becomes new inner_left
            // New point becomes new inner_right
            self.bracket.shrink_left();
            self.left = self.right;
            self.right = point;
        }
    }
}
```

## State.maybe_update_best

```rust
fn maybe_update_best<F: Fn(f64) -> f64>(
    &mut self,
    point: &Point,
    transform: &F,
    snapshot: Snapshot<I, O>,
) {
    let point_score = transform(point.objective);
    let best_score = transform(self.best_point.objective);

    if point_score < best_score {
        self.best_point = *point;
        self.best_snapshot = snapshot;
    }
}
```

---

## Implementation Steps

### 1. Config — validate at construction

**File: `config.rs`**

- Make fields private (remove `pub`)
- Add `new(max_iters, x_abs_tol, x_rel_tol) -> Result<Self, ConfigError>`
- Add getter methods: `max_iters()`, `x_abs_tol()`, `x_rel_tol()`
- Update `Default` to call `Self::new(...).unwrap()`
- Remove `validate()` method (validation happens in `new()`)

### 2. GoldenBracket — add query methods

**File: `bracket.rs`**

- Add `new_inner_left(&self) -> f64`
- Add `new_inner_right(&self) -> f64`
- Remove `#[allow(dead_code)]` if present (methods will be used)

### 3. State — rewrite

**File: `state.rs`**

- Remove `Pending` enum
- Remove `pending` field from `State`
- Remove `left_score`, `right_score`, `best_score` fields (compute on demand via transform)
- Add `ShrinkDirection` enum (tuple variants)
- Rewrite `new()` to take both points and determine best
- Add getters: `left()`, `right()`, `best_point()`
- Add `next_action(&self, transform) -> ShrinkDirection`
- Add `apply(&mut self, direction, point)`
- Add `maybe_update_best(&mut self, point, transform, snapshot)`
- Update `is_converged()` to use config getters
- Update `into_solution()` signature and implementation
- Remove `set_right_init()`, `next_eval()`, `apply_pending()`

### 4. search() — rewrite

**File: `search.rs`**

- Remove `Driver` import and usage
- Import `evaluate` from `crate::optimization::evaluate`
- Import `EvalError` from `crate::optimization::evaluate`
- Implement init logic per spec (evaluate both, match on results)
- Implement main loop per spec
- Keep `map_eval_error` helper function
- Add helper to build failure events from `EvalError`

### 5. Delete driver.rs

**File: `driver.rs`**

- Delete the file
- Remove `mod driver;` from `golden_section.rs`

### 6. Tests — replace

**File: `tests.rs`**

Replace all tests with new test suite.

#### Basic convergence tests

```rust
use std::convert::Infallible;
use approx::assert_relative_eq;
use twine_core::{Model, OptimizationProblem};
use super::{Config, Status, minimize_unobserved, maximize_unobserved};

/// f(x) = x³ - 4x
/// Local min at x = 2/√3 ≈ 1.1547, f(x) ≈ -3.0792
/// Local max at x = -2/√3 ≈ -1.1547, f(x) ≈ 3.0792
struct Cubic;

impl Model for Cubic {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, x: &f64) -> Result<f64, Self::Error> {
        Ok(x.powi(3) - 4.0 * x)
    }
}

struct UseOutput;

impl OptimizationProblem<1> for UseOutput {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Self::Error> { Ok(x[0]) }
    fn objective(&self, _: &f64, output: &f64) -> Result<f64, Self::Error> { Ok(*output) }
}

#[test]
fn minimize_cubic() {
    let solution = minimize_unobserved(&Cubic, &UseOutput, [-2.0, 2.0], &Config::default())
        .expect("should converge");

    assert_eq!(solution.status, Status::Converged);
    assert_relative_eq!(solution.x, 2.0 / 3.0_f64.sqrt(), epsilon = 1e-6);
}

#[test]
fn maximize_cubic() {
    let solution = maximize_unobserved(&Cubic, &UseOutput, [-2.0, 2.0], &Config::default())
        .expect("should converge");

    assert_eq!(solution.status, Status::Converged);
    assert_relative_eq!(solution.x, -2.0 / 3.0_f64.sqrt(), epsilon = 1e-6);
}
```

#### Observer tests (add incrementally)

- `observer_can_stop_early` — stop after N evaluations
- `assume_worse_steers_search` — mark region as worse, verify search avoids it
- `init_one_fails_recovers_with_assume_worse` — right init fails, observer returns AssumeWorse
- `init_one_fails_stops_early` — right init fails, observer returns StopEarly
- `init_both_fail_errors` — both init evals fail, verify error returned
- `loop_failure_recovers` — model fails in loop, observer returns AssumeWorse

---

## Files Summary

| File | Action |
|------|--------|
| `config.rs` | Modify: private fields, `new()`, getters |
| `bracket.rs` | Modify: add query methods |
| `state.rs` | Rewrite: new structure per spec |
| `search.rs` | Rewrite: inline observer, no Driver |
| `driver.rs` | Delete |
| `golden_section.rs` | Modify: remove `mod driver;` |
| `tests.rs` | Replace: new test suite |
| `event.rs` | No changes |
| `action.rs` | No changes |
| `solution.rs` | No changes |
| `error.rs` | No changes |
