/// The golden ratio: φ = (1 + √5) / 2
const PHI: f64 = 1.618_033_988_749_895;

/// The inverse golden ratio: 1/φ
///
/// This equals φ - 1 due to the golden ratio's unique property.
const INV_PHI: f64 = PHI - 1.0;

/// Golden section search bracket.
///
/// Maintains the outer interval [left, right] and two interior points
/// positioned according to the golden ratio.
#[derive(Debug, Clone, Copy)]
pub(super) struct GoldenBracket {
    /// Outer left bound.
    pub(super) left: f64,

    /// Outer right bound.
    pub(super) right: f64,

    /// Inner left point at `left + (1 - φ⁻¹) * width`.
    pub(super) inner_left: f64,

    /// Inner right point at `left + φ⁻¹ * width`.
    pub(super) inner_right: f64,
}

impl GoldenBracket {
    /// Creates a bracket from bounds with interior points positioned by the golden ratio.
    ///
    /// If the bounds are reversed, they are automatically swapped.
    pub(super) fn new(bracket: [f64; 2]) -> Self {
        let [a, b] = bracket;
        let (left, right) = if a <= b { (a, b) } else { (b, a) };
        let width = right - left;
        let inner_left = left + (1.0 - INV_PHI) * width;
        let inner_right = left + INV_PHI * width;
        Self {
            left,
            right,
            inner_left,
            inner_right,
        }
    }

    /// Returns the width of the current bounds.
    pub(super) fn width(&self) -> f64 {
        self.right - self.left
    }

    /// Shrinks the bounds to `[left, inner_right]` and computes a new `inner_left`.
    ///
    /// The old `inner_left` becomes the new `inner_right`, and a new `inner_left`
    /// is computed using the golden ratio.
    pub(super) fn shrink_right(&mut self) {
        self.right = self.inner_right;
        self.inner_right = self.inner_left;
        self.inner_left = self.left + (1.0 - INV_PHI) * self.width();
    }

    /// Shrinks the bounds to `[inner_left, right]` and computes a new `inner_right`.
    ///
    /// The old `inner_right` becomes the new `inner_left`, and a new `inner_right`
    /// is computed using the golden ratio.
    pub(super) fn shrink_left(&mut self) {
        self.left = self.inner_left;
        self.inner_left = self.inner_right;
        self.inner_right = self.left + INV_PHI * self.width();
    }

    /// Returns x for new `inner_left` after shrinking right (without mutating).
    pub(super) fn new_inner_left(&self) -> f64 {
        let new_right = self.inner_right;
        let new_width = new_right - self.left;
        self.left + (1.0 - INV_PHI) * new_width
    }

    /// Returns x for new `inner_right` after shrinking left (without mutating).
    pub(super) fn new_inner_right(&self) -> f64 {
        let new_left = self.inner_left;
        let new_width = self.right - new_left;
        new_left + INV_PHI * new_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    #[test]
    fn bracket_initialized_with_golden_ratio_points() {
        let bracket = GoldenBracket::new([0.0, 1.0]);

        assert_relative_eq!(bracket.left, 0.0);
        assert_relative_eq!(bracket.right, 1.0);
        assert_relative_eq!(bracket.width(), 1.0);

        // inner_left ≈ 0.382, inner_right ≈ 0.618
        assert_relative_eq!(bracket.inner_left, 1.0 - INV_PHI);
        assert_relative_eq!(bracket.inner_right, INV_PHI);

        // They should divide the interval in golden ratio
        assert_relative_eq!(bracket.inner_left / (1.0 - bracket.inner_left), INV_PHI);
    }

    #[test]
    fn bracket_auto_reverses_if_needed() {
        let bracket = GoldenBracket::new([1.0, -2.0]);

        assert_relative_eq!(bracket.left, -2.0);
        assert_relative_eq!(bracket.right, 1.0);
    }

    #[test]
    fn shrink_left_reuses_point_and_computes_new_inner_right() {
        let mut bracket = GoldenBracket::new([0.0, 1.0]);
        let old_inner_right = bracket.inner_right;

        bracket.shrink_left();

        // Old inner_left becomes new left bound
        assert_relative_eq!(bracket.left, 1.0 - INV_PHI);
        assert_relative_eq!(bracket.right, 1.0);

        // Old inner_right becomes new inner_left
        assert_relative_eq!(bracket.inner_left, old_inner_right);

        // New inner_right maintains golden ratio in new interval
        let new_width = bracket.width();
        assert_relative_eq!(bracket.inner_right, bracket.left + INV_PHI * new_width);
    }

    #[test]
    fn shrink_right_reuses_point_and_computes_new_inner_left() {
        let mut bracket = GoldenBracket::new([0.0, 1.0]);
        let old_inner_left = bracket.inner_left;

        bracket.shrink_right();

        // Old inner_right becomes new right bound
        assert_relative_eq!(bracket.left, 0.0);
        assert_relative_eq!(bracket.right, INV_PHI);

        // Old inner_left becomes new inner_right
        assert_relative_eq!(bracket.inner_right, old_inner_left);

        // New inner_left maintains golden ratio in new interval
        let new_width = bracket.width();
        assert_relative_eq!(
            bracket.inner_left,
            bracket.left + (1.0 - INV_PHI) * new_width
        );
    }
}
