//! Module containing a wrapper class for helping use 3d voronoi noise
//! on a spherical surface.

use cgmath::Vector3;
use cgmath::InnerSpace;

use voronoi::{VoronoiSpace, NearResult};

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

    /// Get four nearest points to the point on the surface where the
    /// passed vector passes through
    pub fn near4(&self, v: Vector3<f64>) -> NearResult {
        self.voronoi.near4(self.surf_pos(v))
    }

    /// Get cell which direction vector passes through from the
    /// sphere origin.
    pub fn cell_indices(&self, v: Vector3<f64>) -> Vector3<i64> {
        self.voronoi.region(self.surf_pos(v))
    }

    // Helper functions

    /// Get position on the sphere surface which has the same direction
    /// from the sphere origin as the passed position.
    pub fn surf_pos(&self, v: Vector3<f64>) -> Vector3<f64> {
        v.normalize() * self.radius
    }
}


#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use surface::*;
    use voronoi::VoronoiSpace;

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
