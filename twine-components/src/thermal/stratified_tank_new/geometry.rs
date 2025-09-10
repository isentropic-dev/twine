use std::f64::consts::PI;

use uom::{
    ConstZero,
    si::f64::{Area, Length, Volume},
};

use super::Adjacent;

/// Tank geometry options.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Geometry {
    /// Vertical cylindrical tank.
    VerticalCylinder {
        /// Internal diameter of the tank.
        diameter: Length,
        /// Internal height of the tank.
        height: Length,
    },
}

/// Geometric properties of a discretized node used during tank construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct NodeGeometry {
    pub(super) area: Adjacent<Area>,
    pub(super) height: Length,
    pub(super) volume: Volume,
}

impl Geometry {
    /// Partitions the tank geometry into `N` node geometries.
    ///
    /// Used during tank creation to derive per-node area, height, and volume.
    ///
    /// # Errors
    ///
    /// Returns an error string if the geometry is invalid.
    /// When called from [`StratifiedTank::new`], these errors are surfaced as
    /// [`StratifiedTankCreationError::Geometry`].
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn into_node_geometries<const N: usize>(self) -> Result<[NodeGeometry; N], String> {
        if N == 0 {
            return Err(format!("node count must be ≥ 1, got {N}"));
        }

        match self {
            Geometry::VerticalCylinder { diameter, height } => {
                if diameter <= Length::ZERO {
                    return Err(format!("diameter must be > 0, got {diameter:?}"));
                }
                if height <= Length::ZERO {
                    return Err(format!("height must be > 0, got {height:?}"));
                }

                let end_area = PI * diameter * diameter * 0.25;

                #[allow(clippy::cast_precision_loss)]
                let node_height = height / N as f64;

                Ok([NodeGeometry {
                    area: Adjacent {
                        bottom: end_area,
                        side: PI * diameter * node_height,
                        top: end_area,
                    },
                    height: node_height,
                    volume: end_area * node_height,
                }; N])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{area::square_meter, length::meter, volume::cubic_meter};

    #[test]
    fn vertical_cylinder_with_two_nodes() {
        let geometry = Geometry::VerticalCylinder {
            diameter: Length::new::<meter>(1.0 / PI.sqrt()), // results in a top/bottom area of 0.25 m²
            height: Length::new::<meter>(2.5),
        };

        let [bottom_node, top_node] = geometry.into_node_geometries().unwrap();

        assert_eq!(
            bottom_node, top_node,
            "All nodes in a vertical cylinder have identical geometry."
        );

        assert_relative_eq!(bottom_node.area.bottom.get::<square_meter>(), 0.25);
        assert_relative_eq!(
            bottom_node.area.side.get::<square_meter>(),
            PI / PI.sqrt() * 1.25 // area = π * D * H
        );
        assert_relative_eq!(bottom_node.area.top.get::<square_meter>(), 0.25);

        assert_relative_eq!(bottom_node.height.get::<meter>(), 1.25);
        assert_relative_eq!(bottom_node.volume.get::<cubic_meter>(), 0.3125);
    }
}
