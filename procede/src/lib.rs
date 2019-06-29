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
extern crate lru_cache;
extern crate num_traits;
extern crate wasm_bindgen;

mod voronoi;
mod surface;
mod tectonic;
mod util;
mod world;

type Vec3 = cgmath::Vector3<f64>;

use wasm_bindgen::prelude::*;

use world::World;


// --------------------------------------------------------------------
// JS Interface Functions


#[wasm_bindgen]
pub fn create_world() -> usize {
    let world: Box<World> = Box::new(World::new(88));

    unsafe { std::mem::transmute(Box::into_raw(world)) }
}


#[wasm_bindgen]
pub fn height(world_pos: usize, x: f64, y: f64, z: f64) -> f64 {
    let world_ptr: *mut World = unsafe { std::mem::transmute(world_pos) };
    let world: &mut World = unsafe { world_ptr.as_mut().unwrap() };

    world.height(Vec3::new(x, y, z))
}


#[wasm_bindgen]
pub fn release_world(world_pos: usize) {
    let world: *mut World = unsafe { std::mem::transmute(world_pos) };
    unsafe { drop(world) }
}


#[cfg(test)]
mod tests {
    use create_world;
    use height;

    #[test]
    fn test_lib_interface_produces_heights_in_expected_range() {
        let mem_pos: usize = create_world();
        let mut mean = 0.0;

        for i in -10..11 {
            for j in -10..11 {
                for k in -10..11 {
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
                }
            }
        }

        mean /= 21.0 * 21.0 * 21.0;

        assert!(mean > -1000.0);
        assert!(mean < 1000.0);
    }
}
