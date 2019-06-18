/// Module containing tectonic plate procedural structs and functions.
///
/// For a given point, first the plate and its neighbors are found.
///

use cgmath::{Vector2, Vector3, Vector4};
use voronoi::*;


/// Highest level tectonic struct. Functions provide access to
/// individual plates.
struct TectonicLayer {
    voronoi: VoronoiSpace,
    radius: f64
}


/// Individual tectonic Plate.
///
/// Corresponds to a single voronoi cell.
struct Plate {
    nucleus: Vector3<f64>,
    nucleus_indices: Vector4<i64>,
    neighbors: Vec<Vector4<i64>>,
    motion: Vector2<f64>
}


/// Struct representing a single triangular polygon, defined by a
/// vertex at the plate center, and two vertices at corners of the
/// voronoi cell.
struct PlatePoly {
    vertices: [PlateVertex]
}


/// Struct representing a point on a plate.
///
/// In practice, this will be either the plate center, or one of
/// its corners.
struct PlateVertex {
    position: Vector3<f64>,
    height: f64
}


// --------------------------------------------------------------------
// Implementations


impl TectonicLayer {
    pub const DEFAULT_REGION_WIDTH: f64 = 1e7;  // 10Mm
    pub const DEFAULT_RADIUS: f64 = 6.357e6;

    pub fn new(seed: u32) -> TectonicLayer {
        TectonicLayer {
            voronoi: VoronoiSpace::new(
                seed,
                Vector3::new(
                    Self::DEFAULT_REGION_WIDTH,
                    Self::DEFAULT_REGION_WIDTH,
                    Self::DEFAULT_REGION_WIDTH,
                )
            ),
            radius: Self::DEFAULT_RADIUS
        }
    }

    /// Gets height at surface position identified by direction vector
    /// from origin.
    pub fn height(&self, v: Vector3<f64>) {
        // TODO
    }

    pub fn plate(&self, v: Vector3<f64>) {
        // TODO
    }
}
