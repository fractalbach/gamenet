use cgmath::{Vector3, Vector4};
use cgmath::InnerSpace;
use cgmath::MetricSpace;

use voronoi::{VoronoiSpace, Cell, Neighbor};

/// Module containing a wrapper class for helping use 3d voronoi noise
/// on a spherical surface.

/// Struct handling retrieval of cells and clusters
struct Surface {
    voronoi: VoronoiSpace,
    radius: f64
}


// --------------------------------------------------------------------


/// Convenience class that specializes voronoi cell access for
/// positions on a spherical surface.
impl Surface {
    fn new(voronoi: VoronoiSpace, radius: f64) -> Surface {
        Surface {
            voronoi,
            radius
        }
    }

    /// Get surface cell.
    ///
    /// Finds the SurfaceCell which the passed vector passes through.
    fn cell(&self, v: Vector3<f64>) -> Cell {
        let v_cell = self.voronoi.cell(v.normalize() * self.radius);
        let mut neighbors = Vec::new();
        for v_neighbor in &v_cell.neighbors {
            // If neighbor's surface position (normalized pos * radius)
            // is closer to another neighbor or the cell than it is to
            // its nucleus, then it does not influence the cell's
            // boundary at the surface and it may be discarded.
            let surface_pos = v_neighbor.nucleus.normalize() * self.radius;
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
}
