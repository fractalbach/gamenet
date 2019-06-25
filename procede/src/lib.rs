/// # Procede
///
/// Generates procedural terrain. Called from client and server to
/// produce height and terrain info.
///
/// ## Levels:
///
/// ### Tectonic:
///     For a given point...
///         * Perform gaussian blurring from N sample points:
///             * get base height from nearest plate nuclei.
///             * get ridge / trench height mod from distance to
///                 cell border.
///         * cache?
///
/// ### River:
///     For a given surface point...
///         * Get cell nuclei.
///         * Check for cached map.
///             * If not cached:
///             * Get
///
/// ### SubRiver:
///
///
extern crate cgmath;
extern crate num_traits;
extern crate lru_cache;

mod voronoi;
mod surface;
mod tectonic;
mod util;

type Vec3 = cgmath::Vector3<f64>;

// --------------------------------------------------------------------
// Traits


trait HeightGen {
    fn height(&self, v: Vec3) -> f64;
}

trait NormalGen {
    fn normal(&self, v: Vec3) -> Vec3;
}

trait Layer {
    fn height(&self, h0: f64, v: Vec3) -> f64;
}


// --------------------------------------------------------------------
// Implementations


struct World {
    layers: [Box<Layer>]
}


struct BasicLayer {
    noise: noise::Perlin
}


// World --------------------------------------------------------------


// BasicLayer ---------------------------------------------------------
