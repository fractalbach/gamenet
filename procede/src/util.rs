
use std::num::Wrapping;

use cgmath::{Vector2, Vector3, Vector4};
use cgmath::InnerSpace;
use cgmath::MetricSpace;
use probability::distribution::Continuous;

///
pub fn idx_hash(x: i64) -> u32 {
    let x = Wrapping(x as u32);

    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = (x >> 16) ^ x;

    return x.0;
}


pub fn rand1(x: u32) -> f64 {
    x as f64 / 4294967296.0
}


/// Produces a Vector2<f64> from a random u32.
///
/// Produced x and y values are all between 0 and 1.
pub fn rand2(x: u32) -> Vector2<f64> {
    Vector2::new(
        ((x & 0xFFFF) as f64) / 65536.0,
        ((x ^ 0xFFFF) as f64) / 65536.0,
    )
}


/// Produces a Vector3<f64> from a random u32.
///
/// Produced x, y, and z values are all between 0 and 1.
pub fn rand3(x: u32) -> Vector3<f64> {
    Vector3::new(
        ((x & 0x7FF) as f64) / 2048.0,
        ((x & (0x3FF << 11)) as f64) / 1024.0,
        ((x & (0x7FF << 21)) as f64) / 2048.0
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
pub fn hash_indices(seed: u32, indices: Vector4<i64>) -> u32 {
    let seed_hash = Wrapping(idx_hash(seed as i64));
    let x_hash = Wrapping(idx_hash(indices.x));
    let y_hash = Wrapping(idx_hash(indices.y));
    let z_hash = Wrapping(idx_hash(indices.z));
    let w_hash = Wrapping(idx_hash(indices.w));
    let hash: u32 = (seed_hash + w_hash + x_hash + y_hash + z_hash).0;

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
    let v_norm = v.normalize();
    let u_vec = v_norm.cross(Vector3::new(0.0, 0.0, 1.0)).normalize();
    let v_vec = v_norm.cross(u_vec);

    let step_d = sigma / 3.0;
    let gauss = probability::distribution::Gaussian::new(0.0, sigma);

    let mut h_sum = 0.0;
    let mut density_sum = 0.0;

    for u_i in -2..3 {
        for v_i in -2..3 {
            let u_offset = u_vec * u_i as f64 * step_d;
            let v_offset = v_vec * v_i as f64 * step_d;
            let pos = v + u_offset + v_offset;
            let dist = v.distance2(pos);
            let density = gauss.density(dist);
            h_sum += f(pos) * density;
            density_sum += density;
        }
    }

    h_sum / density_sum
}
