
use std::num::Wrapping;

use cgmath::{Vector2, Vector3, Vector4};

///
pub fn idx_hash(x: i64) -> u32 {
    let x = Wrapping(x as u32);

    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = (x >> 16) ^ x;

    return x.0;
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


pub fn hash_indices(seed: u32, indices: Vector4<i64>) -> u32 {
    let seed_hash = Wrapping(idx_hash(seed as i64));
    let x_hash = Wrapping(idx_hash(indices.x));
    let y_hash = Wrapping(idx_hash(indices.y));
    let z_hash = Wrapping(idx_hash(indices.z));
    let w_hash = Wrapping(idx_hash(indices.w));
    let hash: u32 = (seed_hash + w_hash + x_hash + y_hash + z_hash).0;

    hash
}
