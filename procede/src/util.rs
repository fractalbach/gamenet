
use std::num::Wrapping;

use cgmath::{Vector2, Vector3, Vector4};
use cgmath::InnerSpace;
use cgmath::MetricSpace;

// --------------------------------------------------------------------


/// Hashes a single cell index to produce a new u32.
pub fn idx_hash(x: i64) -> u32 {
    let x = Wrapping(x as u32);

    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = (x >> 16) ^ x;

    return x.0;
}


/// Produces random float from u32.
///
/// Produced values are between -1.0 and 1.0.
pub fn rand1(x: u32) -> f64 {
    x as f64 / 2147483648.0 - 1.0
}


/// Produces a Vector2<f64> from a random u32.
///
/// Produced x and y values are all between -1.0 and 1.0.
pub fn rand2(x: u32) -> Vector2<f64> {
    Vector2::new(
        ((x & 0xFFFF) as f64) / 32768.0 - 1.0,
        (((x >> 16) & 0xFFFF) as f64) / 32768.0 - 1.0,
    )
}


/// Produces a Vector3<f64> from a random u32.
///
/// Produced x, y, and z values are all between -1.0 and 1.0.
pub fn rand3(x: u32) -> Vector3<f64> {
    Vector3::new(
        ((x & 0x7FF) as f64) / 1024.0 - 1.0,
        (((x >> 11) & 0x3FF) as f64) / 512.0 - 1.0,
        (((x >> 21) & 0x7FF) as f64) / 1024.0 - 1.0
    )
}


/// Multiply vectors component-wise.
pub fn component_multiply(a: Vector3<f64>, b: Vector3<f64>) -> Vector3<f64> {
    Vector3::new(
        a.x * b.x,
        a.y * b.y,
        a.z * b.z
    )
}


/// Gets u32 hash of passed cell indices Vector4, and combines it
/// with the passed seed.
pub fn hash_indices(seed: u32, indices: Vector3<i64>) -> u32 {
    let seed_hash = Wrapping(idx_hash(seed as i64));
    let x_hash = Wrapping(idx_hash(indices.x));
    let y_hash = Wrapping(idx_hash(indices.y));
    let z_hash = Wrapping(idx_hash(indices.z));
    let hash: u32 = (seed_hash + x_hash + y_hash + z_hash).0;

    hash
}


/// Gaussian height blur.
///
/// Intended to be used to blur heights in a map.
pub fn hg_blur(
        v: Vector3<f64>,
        sigma: f64,
        f: &mut FnMut (Vector3<f64>) -> f64,
) -> f64 {
    assert_ne!(v, Vector3::new(0.0, 0.0, 0.0));

    let z_axis_vector = Vector3::new(0.0, 0.0, 1.0);
    let v_norm = v.normalize();

    // Get u_vec and v_vec.
    let u_vec: Vector3<f64>;
    if v_norm == z_axis_vector || v_norm == z_axis_vector * -1.0 {
        u_vec = Vector3::new(0.0, 1.0, 0.0);
    } else {
        u_vec = v_norm.cross(z_axis_vector).normalize();
    }
    let v_vec = v_norm.cross(u_vec);

    let step_d = sigma;

    let mut h_sum = 0.0;
    let mut density_sum = 0.0;

    for u_i in -2..3 {
        for v_i in -2..3 {
            let u_offset = u_vec * u_i as f64 * step_d;
            let v_offset = v_vec * v_i as f64 * step_d;
            let pos = v + u_offset + v_offset;
            let dist = v.distance2(pos);
            // TODO: Replace below line with gaussian density.
            let density = 1.0 - dist / sigma * 3.0;
            h_sum += f(pos) * density;
            density_sum += density;
        }
    }

    h_sum / density_sum
}


#[cfg(test)]
mod tests {
    use cgmath::{Vector4, Vector3, Vector2};
    use util::{idx_hash, hash_indices, rand2, rand3, component_multiply};

    #[test]
    fn test_component_wise_vector_multiplication() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(2.0, 3.0, 4.0);

        let r = component_multiply(a, b);

        assert_eq!(r.x, 2.0);
        assert_eq!(r.y, 6.0);
        assert_eq!(r.z, 12.0);
    }

    #[test]
    fn test_idx_hash() {
        let mut mean = 0u32;
        for i in -100..100 {
            mean += idx_hash(i) / 201;
        }

        assert!(mean > (2147483648.0 * 0.8) as u32);
        assert!(mean < (2147483648.0 * 1.2) as u32);
    }

    #[test]
    fn test_hash_indices() {
        let mut mean = 0u32;

        let n_hashes = 10 * 10 * 10 * 5;

        for i in -5..5 {
            for j in -5..5 {
                for k in -5..5 {
                    for seed in 0..5 {
                        mean += hash_indices(
                            seed, Vector3::new(i, j, k)
                        ) / n_hashes;
                    }
                }
            }
        }

        assert!(mean > (2147483648.0 * 0.8) as u32);
        assert!(mean < (2147483648.0 * 1.2) as u32);
    }

    #[test]
    fn test_rand2_produces_results_in_range() {
        let mut mean = Vector2::new(0.0, 0.0);

        for i in 0..1000 {
            let hash = idx_hash(i);

            let rand_vec = rand2(hash);

            assert!(rand_vec.x >= -1.0);
            assert!(rand_vec.y >= -1.0);
            assert!(rand_vec.x <= 1.0);
            assert!(rand_vec.y <= 1.0);

            mean += rand_vec / 1000.0;
        }

        assert!(mean.x > -0.2);
        assert!(mean.x < 0.2);
        assert!(mean.y > -0.2);
        assert!(mean.y < 0.2);
    }

    #[test]
    fn test_rand3_produces_results_in_range() {
        let mut mean = Vector3::new(0.0, 0.0, 0.0);

        for i in 0..1000 {
            let hash = idx_hash(i);

            let rand_vec = rand3(hash);

            assert!(rand_vec.x >= -1.0);
            assert!(rand_vec.y >= -1.0);
            assert!(rand_vec.z >= -1.0);
            assert!(rand_vec.x <= 1.0);
            assert!(rand_vec.y <= 1.0);
            assert!(rand_vec.z <= 1.0);

            mean += rand_vec / 1000.0;
        }

        assert!(mean.x > -0.2);
        assert!(mean.x < 0.2);
        assert!(mean.y > -0.2);
        assert!(mean.y < 0.2);
        assert!(mean.z > -0.2);
        assert!(mean.z < 0.2);
    }
}
