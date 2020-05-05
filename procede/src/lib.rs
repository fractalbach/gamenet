//! # Procede
//!
//! Generates procedural terrain. Called from client and server to
//! produce height and terrain info.
//!
//! ## Levels:
//!
//! ### Tectonic:
//!     * For a given point...
//!         * get base height from nearest plate nuclei.
//!         * get ridge / trench height mod from distance to
//!             cell border.
//!         * cache?
//!
//! ### River:
//!     * For a given surface point...
//!         * Get tectonic cell indices
//!         * Get cell nuclei.
//!         * Check for cached map.
//!             * If not cached:
//!             * Create river graph
//!                 * Create delaunay graph
//!                 * Get Delaunay centers
//!                 * Check if inside tectonic cell or in body of water
//!                         and adjacent to cell.
//!                     * If underwater and adjacent, mark as mouth.
//!                 * Connect to neighbors
//!             * propagate uphill from river mouths (Randomly?)
//!             * Set Strahler number for each point.
//!         * Find nearest river edge (between nodes).
//!         * Set height as function of distance to nearest edge.
//!             (and Strahler number?)
//!
//! ### SubRiver:
//!     * Adds smaller streams to landscape.
//!
//! ### HardProps:
//!     * Adds major static objects that are not moved by settlements
//!             or roads.
//!
//! ### Settlement:
//!     * Generates town & street map.
//!
//! ### Town:
//!     * Generates street & building map for towns.
//!
//! ### SoftProps:
//!     * Adds small or movable static objects whose placement is
//!             potentially affected by towns or roads.
//!

extern crate assert_approx_eq;
extern crate cgmath;
extern crate delaunay2d;
extern crate fnv;
extern crate geo;
extern crate geo_booleanop;
#[macro_use] extern crate geo_types;
#[macro_use] extern crate itertools;
extern crate lru_cache;
extern crate lyon_geom;
extern crate noise;
extern crate num_traits;
extern crate petgraph;
extern crate serde;
extern crate serde_json;
extern crate wasm_bindgen;

#[macro_use]
mod test_util;

mod delaunay;
mod layer;
mod pop;
mod quad;
mod river;
mod surface;
mod tectonic;
mod util;
mod voronoi3;
mod world;

type Vec3 = cgmath::Vector3<f64>;

use wasm_bindgen::prelude::*;

use world::World;


// --------------------------------------------------------------------
// JS Interface Functions


/// Create world instance and return its position in memory as a usize
/// to allow it to be returned to js.
#[wasm_bindgen]
pub fn create_world() -> usize {
    let world: Box<World> = Box::new(World::new(13));

    unsafe { std::mem::transmute(Box::into_raw(world)) }
}


/// Retrieve height of world at the specified position.
#[wasm_bindgen]
pub fn height(world_pos: usize, x: f64, y: f64, z: f64) -> f64 {
    let world_ptr: *mut World = unsafe { std::mem::transmute(world_pos) };
    let world: &mut World = unsafe { world_ptr.as_mut().unwrap() };

    let h = world.height(Vec3::new(x, y, z));
    assert_eq!(h, h);  // Check h is not NaN.
    h
}


/// Release world instance at memory position identified by usize.
#[wasm_bindgen]
pub fn release_world(world_pos: usize) {
    let world: *mut World = unsafe { std::mem::transmute(world_pos) };
    drop(world)
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use ::{create_world, release_world};
    use height;

    /// Checks that a random sampling of points across the surface all
    /// have heights within the expected range.
    #[test]
    fn test_lib_interface_produces_heights_in_expected_range() {
        let mem_pos: usize = create_world();
        let mut mean = 0.0;
        let mut abs_mean = 0.0;

        for (i, j, k) in iproduct!(-10..11, -10..11, -10..11) {
            if i == 0 && j == 0 && k == 0 {
                continue;
            }

            let x = i as f64 / 10.0;
            let y = j as f64 / 10.0;
            let z = k as f64 / 10.0;

            let h = height(mem_pos, x, y, z);

            assert!(h > -15_000.0);
            assert!(h < 15_000.0);

            mean += h;
            abs_mean += h.abs()
        }

        release_world(mem_pos);

        let n_samples = 21.0 * 21.0 * 21.0 - 1.0;
        mean /= n_samples;
        abs_mean /= n_samples;

        assert!(mean > -2000.0);
        assert!(mean < 1000.0);

        assert!(abs_mean > 100.0);
    }

    #[test]
    fn test_height_changes_gradually() {
        let mem_pos: usize = create_world();

        let mut h1 = height(mem_pos, 6.3e6, 0.0, 0.0);
        for i in 1..1000 {
            let h2 = height(mem_pos, 6.3e6, 0.0, i as f64);
            let diff = h2 - h1;
            h1 = h2;

            assert!(diff.abs() < 2.0);
        }

        release_world(mem_pos);
    }
}
