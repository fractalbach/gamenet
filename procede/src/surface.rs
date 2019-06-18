use cgmath::Vector3;
use cgmath::Vector4;

use voronoi::VoronoiSpace;

/// Module containing a wrapper class for helping use 3d voronoi noise
/// on a spherical surface.

/// Struct handling retrieval of cells and clusters
struct Surface {
    voronoi: VoronoiSpace,
    radius: f64
}


struct SurfaceCell {
    nuclei: Vector3<f64>,
    index_vec: Vector4<i64>,
    neighbors: Vec<Vector4<i64>>
}


// --------------------------------------------------------------------


impl Surface {
    fn new(voronoi: VoronoiSpace, radius: f64) -> Surface {
        Surface {
            voronoi,
            radius
        }
    }

    fn cell(v: Vector3<f64>) {
        // todo
    }
}
