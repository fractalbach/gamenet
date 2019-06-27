use cgmath::{Vector3, Vector4};
use cgmath::InnerSpace;
use cgmath::MetricSpace;

use voronoi::{VoronoiSpace, Cell, Neighbor};

/// Module containing a wrapper class for helping use 3d voronoi noise
/// on a spherical surface.

/// Struct handling retrieval of cells and clusters
pub struct Surface {
    voronoi: VoronoiSpace,
    radius: f64
}


// --------------------------------------------------------------------


/// Convenience class that specializes voronoi cell access for
/// positions on a spherical surface.
impl Surface {
    pub fn new(voronoi: VoronoiSpace, radius: f64) -> Surface {
        Surface {
            voronoi,
            radius
        }
    }

    /// Get surface cell.
    ///
    /// Finds the SurfaceCell which the passed vector passes through.
    pub fn cell(&self, v: Vector3<f64>) -> Cell {
        let v_cell = self.voronoi.cell(self.surf_pos(v));
        let mut neighbors = Vec::new();
        for v_neighbor in &v_cell.neighbors {
            // If neighbor's surface position (normalized pos * radius)
            // is closer to another neighbor or the cell than it is to
            // its nucleus, then it does not influence the cell's
            // boundary at the surface and it may be discarded.
            let surface_pos = self.surf_pos(v_neighbor.nucleus);
            let d = v_neighbor.nucleus.distance2(surface_pos);
            if v_cell.nucleus.distance2(surface_pos) < d {
                continue;
            }
            for other_neighbor in &v_cell.neighbors {
                if other_neighbor.indices == v_neighbor.indices {
                    continue;
                }
                if other_neighbor.nucleus.distance2(surface_pos) < d {
                    continue;
                }
                neighbors.push(v_neighbor.clone());
            }
        }

        Cell {
            neighbors,
            ..v_cell
        }
    }

    /// Get cell which direction vector passes through from the
    /// sphere origin.
    pub fn cell_indices(&self, v: Vector3<f64>) -> Vector4<i64> {
        self.voronoi.cell_indices(self.surf_pos(v))
    }

    // Helper functions

    /// Get position on the sphere surface which has the same direction
    /// from the sphere origin as the passed position.
    fn surf_pos(&self, v: Vector3<f64>) -> Vector3<f64> {
        v.normalize() * self.radius
    }
}


#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use surface::*;
    use voronoi::*;

    #[test]
    fn test_different_indices_are_produced_from_different_vectors() {
        const WIDTH: f64 = 1e7;  // 10Mm
        const RADIUS: f64 = 6.357e6;

        let surface = Surface::new(
            VoronoiSpace::new(
                88,
                Vector3::new(
                    WIDTH,
                    WIDTH,
                    WIDTH,
                )
            ),
            RADIUS,
        );
        let a = surface.cell_indices(Vector3::new(1.0, 2.0, -3.0));
        let b = surface.cell_indices(Vector3::new(1.0, 2.0, 3.0));
        let c = surface.cell_indices(Vector3::new(-1.0, -2.0, 3.0));

        assert_ne!(a, b);
        assert_ne!(a, c);
    }
}
