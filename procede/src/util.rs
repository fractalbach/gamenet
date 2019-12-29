
use std::cmp::Ordering;
use std::num::Wrapping;

use cgmath::{Vector2, Vector3, vec2, vec3};
use cgmath::InnerSpace;


pub struct TangentPlane {
    origin: Vector3<f64>
}

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
    vec2(
        ((x & 0xFFFF) as f64) / 32768.0 - 1.0,
        (((x >> 16) & 0xFFFF) as f64) / 32768.0 - 1.0,
    )
}


/// Produces a Vector3<f64> from a random u32.
///
/// Produced x, y, and z values are all between -1.0 and 1.0.
pub fn rand3(x: u32) -> Vector3<f64> {
    vec3(
        ((x & 0x7FF) as f64) / 1024.0 - 1.0,
        (((x >> 11) & 0x3FF) as f64) / 512.0 - 1.0,
        (((x >> 21) & 0x7FF) as f64) / 1024.0 - 1.0
    )
}


/// Multiply vectors component-wise.
pub fn component_multiply(a: Vector3<f64>, b: Vector3<f64>) -> Vector3<f64> {
    vec3(
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


/// Finds U and V vectors for position on surface of a sphere.
///
/// # Arguments
/// * `v` - Vector identifying position on surface of a sphere.
///             This value does not need to be normalized.
///
/// # Returns
/// * u vector - Vector tangential to the surface, aligned with the
///             longitude line of the passed position.
///             (side to side)
///             Will be normalized (magnitude 1).
/// * v vector - Vector tangential to the surface, pointing towards
///             the north pole.
///             Will be normalized (magnitude 1).
pub fn sphere_uv_vec(v: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    assert_ne!(v, vec3(0.0, 0.0, 0.0));

    let z_axis_vector = vec3(0.0, 0.0, 1.0);
    let v_norm = v.normalize();

    // Get u_vec and v_vec.
    let u_vec: Vector3<f64>;
    if v_norm == z_axis_vector || v_norm == z_axis_vector * -1.0 {
        u_vec = vec3(0.0, 1.0, 0.0);
    } else {
        u_vec = z_axis_vector.cross(v_norm).normalize();
    }
    let v_vec = v_norm.cross(u_vec);

    (u_vec, v_vec)
}


/// Determine whether 2d vector a is clockwise from 2d vector b.
///
/// # Arguments
/// * `a` - Vector2 identifying position a.
/// * `b` - Vector2 identifying position b.
///
/// # Returns
/// * true if a is < 1pi/180deg clockwise from b. Otherwise false.
pub fn is_clockwise(a: Vector2<f64>, b: Vector2<f64>) -> bool {
    let acute_cw = b.y * a.x > b.x * a.y;  // true if acute and clockwise.
    if a.dot(b) > 0.0 {acute_cw} else {!acute_cw}
}


/// Convert vector with 3 f64 elements to an array of 3 f64s.
pub fn vec2arr(v: Vector3<f64>) -> [f64; 3] {
    [v.x, v.y, v.z]
}


pub fn clockwise_cmp(a: Vector2<f64>, b: Vector2<f64>) -> Ordering {
    use self::Ordering::{Less, Greater, Equal};

    let a = a.normalize();
    let b = b.normalize();

    if a.x >= 0.0 && b.x < 0.0 {
        return Ordering::Less;
    }
    if a.x < 0.0 && b.x >= 0.0 {
        return Ordering::Greater;
    }
    if a.x == 0.0 && b.x == 0.0 {
        if a.y >= 0.0 && b.y < 0.0 {
            return Ordering::Less
        } else if a.y < 0.0 && b.y > 0.0 {
            return Ordering::Greater
        } else {
            return Ordering::Equal
        }
    }

    // Compare cross-product.
    let det = a.x * b.y - b.x * a.y;
    if det < 0.0 {
        return Ordering::Less;
    }
    if det > 0.0 {
        return Ordering::Greater;
    }

    Ordering::Equal
}


impl TangentPlane {
    pub fn new(origin: Vector3<f64>) -> TangentPlane {
        TangentPlane { origin }
    }

    /// Generates xyz position from a uv position on the TangentPlane
    ///
    /// # Arguments
    /// * `uv` - Position relative to the origin on the TangentPlane.
    ///
    /// # Returns
    /// XYZ Vector3 of position in world-space. Not normalized.
    pub fn xyz(&self, uv: Vector2<f64>) -> Vector3<f64> {
        let (u_vec, v_vec) = sphere_uv_vec(self.origin);
        u_vec * uv.x + v_vec * uv.y + self.origin
    }

    pub fn uv(&self, xyz: Vector3<f64>) -> Vector2<f64> {
        assert!(false);
        vec2(0.0, 0.0)
    }
}


#[cfg(test)]
mod tests {
    use std::cmp::Ordering::{Less, Greater, Equal};

    use cgmath::{Vector3, Vector2, vec2, vec3};
    use util::*;

    #[test]
    fn test_component_wise_vector_multiplication() {
        let a = vec3(1.0, 2.0, 3.0);
        let b = vec3(2.0, 3.0, 4.0);

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
                            seed, vec3(i, j, k)
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
        let mut mean = vec2(0.0, 0.0);

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
        let mut mean = vec3(0.0, 0.0, 0.0);

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

    #[test]
    fn test_sphere_uv_vec_basic_use() {
        let v = vec3(0.3, 0.1, 0.6);
        let (u_vec, v_vec) = sphere_uv_vec(v);

        assert_gt!(u_vec.y, 0.0);
        assert_eq!(u_vec.z, 0.0);
        assert_gt!(v_vec.z, 0.0);
    }

    #[test]
    fn test_sphere_uv_vec_basic_use2() {
        let v = vec3(0.3, 0.1, -0.6);
        let (u_vec, v_vec) = sphere_uv_vec(v);

        assert_gt!(u_vec.y, 0.0);
        assert_eq!(u_vec.z, 0.0);
        assert_gt!(v_vec.z, 0.0);
    }

    #[test]
    fn test_sphere_uv_vec_at_north_pole() {
        let v = vec3(0.0, 0.0, 1.0);
        let (u_vec, v_vec) = sphere_uv_vec(v);

        assert!(!u_vec.x.is_nan());
        assert!(!u_vec.y.is_nan());
        assert!(!v_vec.x.is_nan());
        assert!(!v_vec.y.is_nan());
    }

    #[test]
    fn test_sphere_uv_vec_at_south_pole() {
        let v = vec3(0.0, 0.0, -1.0);
        let (u_vec, v_vec) = sphere_uv_vec(v);

        assert!(!u_vec.x.is_nan());
        assert!(!u_vec.y.is_nan());
        assert!(!v_vec.x.is_nan());
        assert!(!v_vec.y.is_nan());
    }

    #[test]
    fn test_sphere_uv_vec_at_equator() {
        let v = vec3(-1.0, 0.1, -0.0);
        let (u_vec, v_vec) = sphere_uv_vec(v);

        assert_lt!(u_vec.x, 0.0);
        assert_lt!(u_vec.y, 0.0);
        assert_eq!(u_vec.z, 0.0);
        assert_gt!(v_vec.z, 0.0);
    }

    #[test]
    fn test_is_clockwise_basic_cw_case() {
        assert!(is_clockwise(
            vec2(1.0, 1.0), vec2(0.5, 1.0))
        );
    }

    #[test]
    fn test_is_clockwise_basic_ccw_case() {
        assert!(!is_clockwise(
            vec2(-1.0, 2.0), vec2(1.0, 1.0))
        );
    }

    #[test]
    fn test_is_clockwise_basic_obtuse_cw_case() {
        assert!(is_clockwise(
            vec2(0.0, 2.0), vec2(1.0, -1.0))
        );
    }

    #[test]
    fn test_is_clockwise_basic_obtuse_ccw_case() {
        assert!(!is_clockwise(
            vec2(-1.0, 2.0), vec2(-1.0, -1.0))
        );
    }

    #[test]
    fn test_clockwise_cmp_basic_cw_right() {
        assert_eq!(clockwise_cmp(vec2(0.1, 1.0), vec2(1.0, 1.0)), Less);
    }

    #[test]
    fn test_clockwise_cmp_basic_ccw_right() {
        assert_eq!(clockwise_cmp(vec2(1.1, 1.0), vec2(1.0, 1.0)), Greater);
    }

    #[test]
    fn test_clockwise_cmp_basic_cw_left() {
        assert_eq!(clockwise_cmp(vec2(-0.1, 1.0), vec2(-1.0, 1.0)), Greater);
    }

    #[test]
    fn test_clockwise_cmp_basic_ccw_left() {
        assert_eq!(clockwise_cmp(vec2(-1.1, 1.0), vec2(-1.0, 1.0)), Less);
    }

    #[test]
    fn test_clockwise_cmp_basic_equal() {
        assert_eq!(clockwise_cmp(vec2(1.1, 0.5), vec2(2.2, 1.0)), Equal);
    }

    #[test]
    fn test_clockwise_cmp_opposite_x_axis_cw() {
        assert_eq!(clockwise_cmp(vec2(0.7, 0.7), vec2(-0.7, 0.7)), Less);
    }

    #[test]
    fn test_clockwise_cmp_opposite_x_axis_ccw() {
        assert_eq!(clockwise_cmp(vec2(-0.7, 0.7), vec2(0.7, 0.7)), Greater);
    }

    #[test]
    fn test_clockwise_cmp_on_x_axis_cw() {
        assert_eq!(clockwise_cmp(vec2(0.0, 0.7), vec2(0.0, -1.0)), Less);
    }

    #[test]
    fn test_clockwise_cmp_on_x_axis_ccw() {
        assert_eq!(clockwise_cmp(vec2(0.0, -1.0), vec2(0.0, 1.0)), Greater);
    }
}
