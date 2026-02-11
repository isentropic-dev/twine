use twine_core::Snapshot;

use super::bracket::GoldenBracket;
use super::solution::Status;
use super::{Config, Point, Solution};

/// Direction to shrink the bracket and where to evaluate next.
#[derive(Debug, Clone, Copy)]
pub(super) enum ShrinkDirection {
    /// Shrink left bound; payload is x for new `inner_right`.
    ShrinkLeft(f64),

    /// Shrink right bound; payload is x for new `inner_left`.
    ShrinkRight(f64),
}

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
    ) -> Self {
        Self {
            bracket,
            left,
            right,
            best_point,
            best_snapshot,
        }
    }

    pub(super) fn left(&self) -> Point {
        self.left
    }

    pub(super) fn right(&self) -> Point {
        self.right
    }

    /// Pure query: which direction to shrink and where to evaluate next.
    pub(super) fn next_action<F: Fn(f64) -> f64>(&self, transform: &F) -> ShrinkDirection {
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

    /// Apply shrink and update interior point with new evaluation.
    pub(super) fn apply(&mut self, direction: ShrinkDirection, point: Point) {
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

    /// Update best if this point has better score. Only call with real evaluations.
    pub(super) fn maybe_update_best<F: Fn(f64) -> f64>(
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

    pub(super) fn is_converged(&self, config: &Config) -> bool {
        let gap = (self.bracket.inner_right - self.bracket.inner_left).abs();
        let mid = 0.5 * (self.bracket.inner_left + self.bracket.inner_right);
        let x_ref = mid.abs();
        gap <= config.x_abs_tol() + config.x_rel_tol() * x_ref
    }

    pub(super) fn into_solution(self, status: Status, iters: usize) -> Solution<I, O> {
        Solution {
            status,
            x: self.best_point.x,
            objective: self.best_point.objective,
            snapshot: self.best_snapshot,
            iters,
        }
    }
}
