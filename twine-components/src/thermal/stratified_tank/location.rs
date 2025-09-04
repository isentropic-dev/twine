use uom::{ConstZero, si::f64::Length};

#[derive(Debug, Clone, Copy)]
pub struct Location {
    center: Position,
    span: Length,
}

impl Location {
    #[must_use]
    pub fn point_abs(z: Length) -> Self {
        Self {
            center: Position::Absolute(z),
            span: Length::ZERO,
        }
    }

    #[must_use]
    pub fn point_rel(frac: f64) -> Self {
        Self {
            center: Position::Relative(frac),
            span: Length::ZERO,
        }
    }

    #[must_use]
    pub fn span_abs(center: Length, span: Length) -> Self {
        Self {
            center: Position::Absolute(center),
            span,
        }
    }

    #[must_use]
    pub fn span_rel(center_frac: f64, span: Length) -> Self {
        Self {
            center: Position::Relative(center_frac),
            span,
        }
    }

    #[must_use]
    pub fn tank_bottom() -> Self {
        Self::point_rel(0.0)
    }

    #[must_use]
    pub fn tank_top() -> Self {
        Self::point_rel(1.0)
    }

    pub(super) fn into_weights<const N: usize>(
        self,
        heights: &[Length; N],
    ) -> Result<[f64; N], String> {
        let Self { center, span } = self;
        let total_height = heights.iter().copied().sum();

        let center = match center {
            Position::Relative(frac) => {
                if !(0.0..=1.0).contains(&frac) {
                    return Err(format!("relative center must be in [0, 1], got {frac}"));
                }
                total_height * frac
            }
            Position::Absolute(z) => z,
        };

        if span < Length::ZERO {
            return Err(format!("span must be ≥ 0, got {span:?}"));
        }

        let node_tops: [Length; N] = {
            let mut acc = Length::ZERO;
            heights.map(|h| {
                acc += h;
                acc
            })
        };

        let mut weights = [0.0; N];
        if span == Length::ZERO {
            // Location is a point.
            if center < Length::ZERO || center > total_height {
                return Err(format!(
                    "location out of bounds: {center:?} not within [0, {total_height:?}]"
                ));
            }

            let index = node_tops
                .partition_point(|&node_top| center >= node_top)
                .min(N - 1);

            weights[index] = 1.0;
        } else {
            // Location has a non-zero span.
            let (z0, z1) = (center - span * 0.5, center + span * 0.5);

            if z0 < Length::ZERO || z1 > total_height {
                return Err(format!(
                    "location out of bounds: [{z0:?}, {z1:?}] not within [0, {total_height:?}]"
                ));
            }

            let mut start = Length::ZERO;
            for i in 0..N {
                let end = node_tops[i];
                let lo = z0.max(start);
                let hi = z1.min(end);
                if hi > lo {
                    weights[i] = ((hi - lo) / span).into();
                }
                start = end;
            }
        }

        Ok(weights)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PortLocation {
    pub inlet: Location,
    pub outlet: Location,
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    /// Fraction of total tank height from bottom.
    Relative(f64),
    /// Absolute distance from bottom.
    Absolute(Length),
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::length::meter;

    fn m(v: f64) -> Length {
        Length::new::<meter>(v)
    }

    #[test]
    fn point_abs_selects_containing_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_abs(m(1.2));

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 1.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_rel_bottom() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_rel(0.0);

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 1.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_rel_top() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_rel(1.0);

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 1.0);
    }

    #[test]
    fn point_on_internal_boundary_maps_to_upper_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_abs(m(1.0));

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 1.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_rel_out_of_range_errors() {
        let heights = [m(1.0), m(1.0)];
        assert!(Location::point_rel(-0.01).into_weights(&heights).is_err());
        assert!(Location::point_rel(1.01).into_weights(&heights).is_err());
    }

    #[test]
    fn point_center_out_of_bounds_errors() {
        let heights = [m(1.0), m(1.0)];
        let err = Location::point_abs(m(2.1))
            .into_weights(&heights)
            .unwrap_err();
        assert!(err.contains("out of bounds"));
    }

    #[test]
    fn span_within_single_node_all_weight_in_that_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        // The span [0.25, 0.75] is entirely in node 0.
        let loc = Location::span_abs(m(0.5), m(0.5));

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 1.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn span_crosses_two_nodes_distributes_by_overlap() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        // The span [0.65, 1.15] overlap 0.35 (70%) in node 0, 0.15 (30%) in node 1.
        let loc = Location::span_abs(m(0.9), m(0.5));

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.70);
        assert_relative_eq!(w1, 0.30);
        assert_relative_eq!(w2, 0.00);
    }

    #[test]
    fn span_over_three_of_five_nodes_distributes_by_overlap() {
        // 5 nodes, each 1 m tall
        // Node ranges: [0,1], [1,2], [2,3], [3,4], [4,5]
        // Location: relative center is 0.5 (2.5 m), span = 2.0 m -> [1.5, 3.5]
        // Overlaps: n0=0, n1=0.5 m, n2=1.0 m, n3=0.5 m, n4=0  (total span = 2.0)
        // Expected weights: [0, 0.25, 0.50, 0.25, 0]
        let heights = [m(1.0), m(1.0), m(1.0), m(1.0), m(1.0)];
        let loc = Location::span_rel(0.5, m(2.0));

        let [w0, w1, w2, w3, w4] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 0.25);
        assert_relative_eq!(w2, 0.50);
        assert_relative_eq!(w3, 0.25);
        assert_relative_eq!(w4, 0.0);
    }

    #[test]
    fn span_full_tank_matches_node_height_fractions() {
        let heights = [m(1.0), m(2.0), m(3.0)];
        let loc = Location::span_rel(0.5, m(6.0));

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 1.0 / 6.0);
        assert_relative_eq!(w1, 2.0 / 6.0);
        assert_relative_eq!(w2, 3.0 / 6.0);
    }

    #[test]
    fn tiny_span_centered_on_boundary_splits_evenly() {
        let heights = [m(0.1), m(0.2)];
        // Span is tiny but > 0, centered exactly at z = 0.1.
        let loc = Location::span_abs(m(0.1), m(1e-6));

        let [w0, w1] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.5, epsilon = 1e-12);
        assert_relative_eq!(w1, 0.5, epsilon = 1e-12);
    }

    #[test]
    fn span_out_of_bounds_errors() {
        let heights = [m(1.0), m(1.0)];
        // band [0.75, 2.25] exceeds top
        let loc = Location::span_abs(m(1.5), m(1.5));
        let err = loc.into_weights(&heights).unwrap_err();
        assert!(err.contains("out of bounds"));
    }

    #[test]
    fn negative_span_errors() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::span_abs(m(0.5), m(-0.1));
        let err = loc.into_weights(&heights).unwrap_err();
        assert!(err.contains("span must be ≥ 0"));
    }
}
