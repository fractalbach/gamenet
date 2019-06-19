/// Module containing tectonic plate procedural structs and functions.
///
/// For a given point, first the plate and its neighbors are found.
///

use cgmath::{Vector2, Vector3, Vector4};
use lru_cache::LruCache;

use voronoi::*;
use surface::Surface;
use util::{rand2, hash_indices};


/// Highest level tectonic struct. Functions provide access to
/// individual plates.
struct TectonicLayer {
    seed: u32,
    surface: Surface,
    cache: LruCache<Vector4<i64>, Plate>,
}


/// Individual tectonic Plate.
///
/// Corresponds to a single voronoi cell.
struct Plate {
    cell: Cell,
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
    pub const DEFAULT_CACHE_SIZE: usize = 1_000;

    pub fn new(seed: u32) -> TectonicLayer {
        TectonicLayer {
            seed,
            surface: Surface::new(
                VoronoiSpace::new(
                    seed,
                    Vector3::new(
                        Self::DEFAULT_REGION_WIDTH,
                        Self::DEFAULT_REGION_WIDTH,
                        Self::DEFAULT_REGION_WIDTH,
                    )
                ),
                Self::DEFAULT_RADIUS,
            ),
            cache: LruCache::new(Self::DEFAULT_CACHE_SIZE)
        }
    }

    /// Gets height at surface position identified by direction vector
    /// from origin.
    pub fn height(&self, v: Vector3<f64>) {
        // TODO
    }

    /// Get Plate for the specified direction from planet center.
    pub fn plate(&mut self, v: Vector3<f64>) -> Option<&mut Plate> {
        let cell_indices = self.surface.cell_indices(v);
        if self.cache.contains_key(&cell_indices) {
            return self.cache.get_mut(&cell_indices);
        }

        let cell = self.surface.cell(v);
        let motion = rand2(hash_indices(self.seed, cell_indices));

        let plate = Plate {
            cell,
            motion,
        };

        self.cache.insert(cell_indices, plate);

        self.cache.get_mut(&cell_indices)
    }

    /// Get indices of plate passed through by passed direction vector
    /// from globe center.
    fn plate_indices(&self, v: Vector3<f64>) -> Vector4<i64> {
        self.surface.cell_indices(v)
    }
}


// --------------------------------------------------------------------



