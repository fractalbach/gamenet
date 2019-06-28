/// Module containing tectonic plate procedural structs and functions.
///

use cgmath::{Vector2, Vector3, Vector4};
use lru_cache::LruCache;

use voronoi::*;
use surface::Surface;
use util::{rand1, rand2, hash_indices, hg_blur};


/// Highest level tectonic struct. Functions provide access to
/// individual plates.
struct TectonicLayer {
    seed: u32,
    surface: Surface,
    cache: LruCache<Vector4<i64>, Plate>,
    min_base_height: f64,
    max_base_height: f64,
    base_height_range: f64,
    blur_radius: f64,
}


/// Individual tectonic Plate.
///
/// Corresponds to a single voronoi cell.
#[derive(Clone)]
struct Plate {
    cell: Cell,
    pub motion: Vector2<f64>,
    pub base_height: f64,
}


// --------------------------------------------------------------------
// Implementations


impl TectonicLayer {
    pub const DEFAULT_REGION_WIDTH: f64 = 1e7;  // 10Mm
    pub const DEFAULT_RADIUS: f64 = 6.357e6;
    pub const DEFAULT_CACHE_SIZE: usize = 1_000;
    pub const DEFAULT_MIN_BASE_HEIGHT: f64 = -2000.0;
    pub const DEFAULT_MAX_BASE_HEIGHT: f64 = 2000.0;
    pub const DEFAULT_BLUR_SIGMA: f64 = 1e5;  // 100km.

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
            cache: LruCache::new(Self::DEFAULT_CACHE_SIZE),
            min_base_height: Self::DEFAULT_MIN_BASE_HEIGHT,
            max_base_height: Self::DEFAULT_MAX_BASE_HEIGHT,
            base_height_range: Self::DEFAULT_MAX_BASE_HEIGHT -
                    Self::DEFAULT_MIN_BASE_HEIGHT,
            blur_radius: Self::DEFAULT_BLUR_SIGMA,
        }
    }

    /// Gets height at surface position identified by direction vector
    /// from origin.
    pub fn height(&mut self, v: Vector3<f64>) -> f64 {
        hg_blur(v, self.blur_radius, &mut |v| {
            self.plate(v).unwrap().base_height
        })
    }

    /// Get Plate for the specified direction from planet center.
    pub fn plate(&mut self, v: Vector3<f64>) -> Option<&mut Plate> {
        let cell_indices = self.surface.cell_indices(v);
        if self.cache.contains_key(&cell_indices) {
            return self.cache.get_mut(&cell_indices);
        }

        let cell = self.surface.cell(v);
        self.cache.insert(cell_indices, Plate::new(self.seed, cell));

        self.cache.get_mut(&cell_indices)
    }

    /// Get indices of plate passed through by passed direction vector
    /// from globe center.
    fn plate_indices(&self, v: Vector3<f64>) -> Vector4<i64> {
        self.surface.cell_indices(v)
    }
}


// --------------------------------------------------------------------


impl Plate {
    fn new(seed: u32, cell: Cell) -> Self {
        let hash = hash_indices(seed, cell.indices);
        let motion = rand2(hash);
        let base_height = rand1(hash);

        Plate {
            cell,
            motion,
            base_height,
        }
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use tectonic::*;
    use voronoi::*;

    #[test]
    fn test_plate_motion_differs() {
        let mut tectonic = TectonicLayer::new(1);
        let motion1 = tectonic.plate(
            Vector3::new(1.0, 2.0, -3.0)
        ).unwrap().motion;
        let motion2 = tectonic.plate(
            Vector3::new(1.0, 2.0, 3.0)
        ).unwrap().motion;
        let motion3 = tectonic.plate(
            Vector3::new(-1.0, -2.0, 3.0)
        ).unwrap().motion;

        assert_ne!(motion1, motion2);
        assert_ne!(motion1, motion3);
    }

    #[test]
    fn test_plate_motion_is_consistent() {
        let mut tectonic = TectonicLayer::new(1);
        let motion1a = tectonic.plate(
            Vector3::new(1.0, 2.0, 3.0)
        ).unwrap().motion;
        let motion1b = tectonic.plate(
            Vector3::new(1.0, 2.0, 3.0)
        ).unwrap().motion;

        assert_eq!(motion1a, motion1b);
    }
}
