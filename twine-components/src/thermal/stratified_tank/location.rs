use uom::{ConstZero, si::f64::Length};

/// Vertical placement for ports and auxiliary sources in the tank.
///
/// A location can be:
/// - `Node(index)`: a discrete node by index,
/// - `Point(position)`: a point given absolutely or relatively,
/// - `Span(position, span)`: a symmetric span around an absolute/relative center.
///
/// Spans distribute weight across nodes in proportion to geometric overlap.
/// A point that lies exactly on an internal boundary maps to the lower node.
///
/// Constructors do not check invariants.
/// Validation occurs when locations are mapped to nodes during tank creation,
/// and invalid locations are reported via `StratifiedTankCreationError`.
#[derive(Debug, Clone, Copy)]
pub enum Location {
    Node(usize),
    Point(Position),
    Span(Position, Length),
}

/// Vertical reference for specifying a position in a tank.
///
/// A position can be expressed in one of two ways:
/// - `Relative(f64)`: a fraction of the total tank height in `[0, 1]`,
/// - `Absolute(Length)`: a physical distance.
#[derive(Debug, Clone, Copy)]
pub enum Position {
    Relative(f64),
    Absolute(Length),
}

impl Location {
    /// Point inside the node at the given index.
    #[must_use]
    pub fn point_in_node(index: usize) -> Self {
        Self::Node(index)
    }

    /// Point at an absolute height.
    #[must_use]
    pub fn point_abs(z: Length) -> Self {
        Self::Point(Position::Absolute(z))
    }

    /// Point at a relative height.
    #[must_use]
    pub fn point_rel(frac: f64) -> Self {
        Self::Point(Position::Relative(frac))
    }

    /// Span centered at an absolute height.
    #[must_use]
    pub fn span_abs(center: Length, span: Length) -> Self {
        Self::Span(Position::Absolute(center), span)
    }

    /// Span centered at a relative height.
    #[must_use]
    pub fn span_rel(center_frac: f64, span: Length) -> Self {
        Self::Span(Position::Relative(center_frac), span)
    }

    /// Point at the bottom of the tank.
    #[must_use]
    pub fn tank_bottom() -> Self {
        Self::point_rel(0.0)
    }

    /// Point at the top of the tank.
    #[must_use]
    pub fn tank_top() -> Self {
        Self::point_rel(1.0)
    }

    pub(super) fn into_weights<const N: usize>(
        self,
        heights: &[Length; N],
    ) -> Result<[f64; N], String> {
        if N == 0 {
            return Err("must have at least one node".into());
        }

        let node_tops: [Length; N] = {
            let mut acc = Length::ZERO;
            heights.map(|h| {
                acc += h;
                acc
            })
        };
        let total_height = node_tops[N - 1];

        let mut weights = [0.0; N];

        match self {
            Location::Node(index) => {
                if index >= N {
                    return Err(format!("node index must be in 0..{N}, got {index}"));
                }
                weights[index] = 1.0;
            }
            Location::Point(position) => {
                let z = position.to_abs(total_height)?;
                if z < Length::ZERO || z > total_height {
                    return Err(format!(
                        "location out of bounds: {z:?} not within [0, {total_height:?}]"
                    ));
                }

                let index = node_tops
                    .partition_point(|&node_top| z > node_top)
                    .min(N - 1);

                weights[index] = 1.0;
            }
            Location::Span(position, span) => {
                let center = position.to_abs(total_height)?;
                if span <= Length::ZERO {
                    return Err(format!("span must be > 0, got {span:?}"));
                }

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
        }

        Ok(weights)
    }
}

/// Port pair placement: inlet and outlet locations.
#[derive(Debug, Clone, Copy)]
pub struct PortLocation {
    pub inlet: Location,
    pub outlet: Location,
}

impl Position {
    fn to_abs(self, total: Length) -> Result<Length, String> {
        match self {
            Position::Absolute(z) => Ok(z),
            Position::Relative(frac) => {
                if (0.0..=1.0).contains(&frac) {
                    Ok(total * frac)
                } else {
                    Err(format!("relative fraction must be in [0, 1], got {frac}"))
                }
            }
        }
    }
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
    fn point_on_internal_boundary_maps_to_lower_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_abs(m(2.0));

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
    fn point_in_node_zero_maps_to_first_node() {
        let heights = [m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_in_node(0);

        let [w0, w1, w2] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 1.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 0.0);
    }

    #[test]
    fn point_in_middle_node_maps_correctly() {
        let heights = [m(1.0), m(1.0), m(1.0), m(1.0), m(1.0)];
        let loc = Location::point_in_node(2);

        let [w0, w1, w2, w3, w4] = loc.into_weights(&heights).unwrap();

        assert_relative_eq!(w0, 0.0);
        assert_relative_eq!(w1, 0.0);
        assert_relative_eq!(w2, 1.0);
        assert_relative_eq!(w3, 0.0);
        assert_relative_eq!(w4, 0.0);
    }

    #[test]
    fn point_in_node_last_index_errors() {
        let heights = [m(1.0), m(1.0)];
        let err = Location::point_in_node(2)
            .into_weights(&heights)
            .unwrap_err();

        assert!(err.contains("node index must be in 0..2"));
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
        // The span [0.65, 1.15] overlaps 0.35 (70%) in node 0, 0.15 (30%) in node 1.
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
    fn zero_and_negative_span_errors() {
        let heights = [m(1.0), m(1.0), m(1.0)];

        let loc = Location::span_abs(m(0.5), m(0.0));
        let err = loc.into_weights(&heights).unwrap_err();
        assert!(err.contains("span must be > 0"));

        let loc = Location::span_abs(m(0.5), m(-0.1));
        let err = loc.into_weights(&heights).unwrap_err();
        assert!(err.contains("span must be > 0"));
    }
}
