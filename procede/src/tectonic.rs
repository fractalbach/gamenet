//! Module containing tectonic plate procedural structs and functions.
//!

use cgmath::{Vector2, Vector3};
use cgmath::InnerSpace;
use lru_cache::LruCache;

use voronoi::*;
use surface::Surface;
use util::{hash_indices, vec2arr};
use noise::{Perlin, Fbm, NoiseFn, Seedable};


// --------------------------------------------------------------------


/// Highest level tectonic struct. Functions provide access to
/// individual plates.
pub struct TectonicLayer {
    pub seed: u32,
    pub surface: Surface,
    cache: LruCache<Vector3<i64>, Plate>,
    h_noise: Fbm,
    x_motion_noise: Fbm,
    y_motion_noise: Fbm,
    pos_noise: Fbm,
    ridge_noise: Perlin,
    min_base_height: f64,
    max_base_height: f64,
    mean_base_height: f64,
    base_height_range: f64,
    max_ridge_height: f64,
    blur_radius: f64,
}

/// Struct used to return height and related info about a position.
#[derive(Clone)]
pub struct TectonicInfo {
    pub height: f64,
    pub indices: Vector3<i64>,
    pub nucleus: Vector3<f64>,
}

/// Individual tectonic Plate.
///
/// Corresponds to a single voronoi cell.
#[derive(Clone)]
struct Plate {
    pub indices: Vector3<i64>,
    pub nucleus: Vector3<f64>,
    hash: u32,
    pub motion: Vector2<f64>,
    pub base_height: f64,
}


// --------------------------------------------------------------------
// Implementations


impl TectonicLayer {
    pub const DEFAULT_REGION_WIDTH: f64 = 3.0e6;
    pub const DEFAULT_RADIUS: f64 = 6.357e6;
    pub const DEFAULT_CACHE_SIZE: usize = 1_000;
    pub const DEFAULT_MIN_BASE_HEIGHT: f64 = -3920.0;
    pub const DEFAULT_MAX_BASE_HEIGHT: f64 = 1680.0;
    pub const DEFAULT_BLUR_SIGMA: f64 = 2e5;  // 200km.
    pub const DEFAULT_MAX_RIDGE_HEIGHT: f64 = 8_000.0;

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
            h_noise: Fbm::new().set_seed(seed),
            x_motion_noise: Fbm::new().set_seed(seed + 100),
            y_motion_noise: Fbm::new().set_seed(seed + 200),
            pos_noise: Fbm::new().set_seed(seed + 300),
            ridge_noise: Perlin::new().set_seed(seed + 400),
            min_base_height: Self::DEFAULT_MIN_BASE_HEIGHT,
            max_base_height: Self::DEFAULT_MAX_BASE_HEIGHT,
            mean_base_height: (
                    Self::DEFAULT_MIN_BASE_HEIGHT +
                    Self::DEFAULT_MAX_BASE_HEIGHT
            ) / 2.0,
            base_height_range: Self::DEFAULT_MAX_BASE_HEIGHT -
                    Self::DEFAULT_MIN_BASE_HEIGHT,
            max_ridge_height: Self::DEFAULT_MAX_RIDGE_HEIGHT,
            blur_radius: Self::DEFAULT_BLUR_SIGMA,
        }
    }

    /// Gets height at surface position identified by direction vector
    /// from origin.
    ///
    /// # Arguments
    /// * `v` - Position Vector3. Does not require normalization:
    ///             Will be mapped to world surface.
    ///
    /// # Return
    /// TectonicInfo with height and plate info for the passed position.
    pub fn height(&mut self, v: Vector3<f64>) -> TectonicInfo {
        let adj_pos = self.adjust_pos(v);
        let near_result = self.surface.near4(adj_pos);

        let nearest_d = near_result.dist[0];

        let mut base_mean = 0.0;
        let mut base_weight_sum = 1.0;
        let mut ridge_mean = 0.0;
        let mut ridge_weight_sum = 0.0;
        let mut nearest_motion = Vector2::new(0.0, 0.0);

        for i in 0..4 {
            // Find distance to edge of cell.
            let edge_dist = (near_result.dist[i] - nearest_d) / 2.0;
            if edge_dist >= self.blur_radius {
                continue;
            }

            // Get plate info
            let plate_base_height: f64;
            let plate_motion: Vector2<f64>;
            {
                let plate = self.plate(
                    near_result.regions[i],
                    near_result.points[i]
                ).unwrap();
                plate_base_height = plate.base_height;
                plate_motion = plate.motion;
            }

            // Find base height and ridge height.
            if i == 0 {
                base_mean += plate_base_height;
                nearest_motion = plate_motion;
            } else {
                let weight = 1.0 - edge_dist / self.blur_radius;
                base_mean += plate_base_height * weight;
                base_weight_sum += weight;
                let (ridge_h, ridge_w) = self.ridge_h(
                    adj_pos,
                    near_result.points[0],
                   nearest_motion,
                   near_result.points[i],
                   plate_motion,
                    edge_dist,
                );
                ridge_mean += ridge_h;
                ridge_weight_sum += ridge_w;
            }
        }
        base_mean /= base_weight_sum;

        let mut h = base_mean;
        if ridge_weight_sum > 0.0 {
            ridge_mean /= ridge_weight_sum;
            h += self.ridge_invert(ridge_mean, base_mean);
        }

        TectonicInfo {
            height: h,
            indices: near_result.regions[0],
            nucleus: near_result.points[0],
        }
    }

    /// Adjust input world position.
    ///
    /// Input vector will be normalized and then modified according to
    /// the tectonic warp noise, and then shrunk on the z axis in order
    /// to more closely resemble real-world tectonic plate behavior.
    ///
    /// # Arguments
    /// * `v` - Position Vector3. Does not require normalization:
    ///             Will be mapped to world surface.
    ///
    /// # Return
    /// Modified position Vector3.
    fn adjust_pos(&self, v: Vector3<f64>) -> Vector3<f64> {
        let v = v.normalize();
        let noise_amp = 0.6;
        let noise_frq = 0.6;
        let x_noise = self.pos_noise.get(vec2arr(v * noise_frq)) * noise_amp;
        let y_noise = self.pos_noise.get(
            vec2arr(v * noise_frq + Vector3::new(0.5, 0.0, 0.0) * noise_frq)
        ) * noise_amp;
        let z_noise = self.pos_noise.get(
            vec2arr(v * noise_frq - Vector3::new(0.7, 0.0, 0.0) * noise_frq)
        ) * noise_amp;

        Vector3::new(
            v.x + x_noise,
            v.y + y_noise,
            (v.z + z_noise) * 0.66,
        )
    }

    /// Get Plate for the specified direction from planet center.
    fn plate(
            &mut self,
            indices: Vector3<i64>,
            nucleus: Vector3<f64>
    ) -> Option<&mut Plate> {
        if self.cache.contains_key(&indices) {
            return self.cache.get_mut(&indices);
        }

        let plate = Plate::new(self.seed, nucleus, indices, &self);
        self.cache.insert(indices, plate);

        self.cache.get_mut(&indices)
    }

    /// Get ridge height of two plates
    fn ridge_h(
        &self,
        v: Vector3<f64>,
        a_nucleus: Vector3<f64>,
        a_motion: Vector2<f64>,
        b_nucleus: Vector3<f64>,
        b_motion: Vector2<f64>,
        edge_dist: f64,
    ) -> (f64, f64) {
        let weight = 1.0 - edge_dist / self.blur_radius;
        let closing_rate = self.closing_rate(
            a_nucleus,
            a_motion,
            b_nucleus,
            b_motion
        );
        assert!(closing_rate.abs() <= 1.0);
        let noise = self.ridge_noise.get(vec2arr(v));
        let amp = weight * (0.8 + sign_safe_sqrt(noise));
        let mut ridge_h = closing_rate * self.max_ridge_height * amp;
        if ridge_h < 0.0 {
            ridge_h /= 3.0;
        }

        (ridge_h, weight)
    }

    fn ridge_invert(&self, ridge_h: f64, base_h: f64) -> f64 {
        if base_h > 1000.0 {
            return ridge_h;
        } else if base_h < -1000.0 {
            return -ridge_h;
        }
        let scale = base_h / 1000.0;
        ridge_h * scale
    }

    /// Get closing rate of two plates.
    fn closing_rate(
        &self,
        a_nucleus: Vector3<f64>,
        a_motion: Vector2<f64>,
        b_nucleus: Vector3<f64>,
        b_motion: Vector2<f64>,
    ) -> f64 {
        let a_surf_pos = self.surface.surf_pos(a_nucleus);
        let b_surf_pos = self.surface.surf_pos(b_nucleus);

        let a_motion3d = Self::lat_lon_2_3d(a_surf_pos, a_motion);
        let b_motion3d = Self::lat_lon_2_3d(b_surf_pos, b_motion);

        let pos_diff = b_surf_pos - a_surf_pos;
        let mot_diff = a_motion3d - b_motion3d;

        let angle_cos = pos_diff.normalize().dot(mot_diff.normalize());
        angle_cos * mot_diff.magnitude() / 2.0
    }

    /// Convert lat/lon motion vector into 3d.
    fn lat_lon_2_3d(p: Vector3<f64>, motion: Vector2<f64>) -> Vector3<f64> {
        let z_axis_vector = Vector3::new(0.0, 0.0, 1.0);
        let v_norm = p.normalize();

        // Get u_vec and v_vec.
        let u_vec: Vector3<f64>;
        if v_norm == z_axis_vector || v_norm == z_axis_vector * -1.0 {
            u_vec = Vector3::new(0.0, 1.0, 0.0);
        } else {
            u_vec = v_norm.cross(z_axis_vector).normalize();
        }
        let v_vec = v_norm.cross(u_vec);

        u_vec * motion.x + v_vec * motion.y
    }
}


// --------------------------------------------------------------------


impl Plate {
    fn new(
            seed: u32,
            nucleus: Vector3<f64>,
            indices: Vector3<i64>,
            layer: &TectonicLayer
    ) -> Self {
        let hash = hash_indices(seed, indices);
        let sample_p = layer.surface.surf_pos(nucleus) / 6.3e6;
        let noise = layer.h_noise.get([
            sample_p.x,
            sample_p.y,
            sample_p.z / 0.66
        ]);

        // Divide by 1.86 instead of 2.0 to make up some of the scale
        // lost by Fbm noise, which often is between -0.5 to 0.5
        //
        // This gets the land/surface ratio closer to what's
        // desired (0.3)
        let base_height = sign_safe_sqrt(noise) *
            layer.base_height_range / 1.86 + layer.mean_base_height;
        let motion = Vector2::new(
            layer.x_motion_noise.get(vec2arr(sample_p)),
            layer.y_motion_noise.get(vec2arr(sample_p)),
        );

        Plate {
            indices,
            nucleus,
            hash,
            motion,
            base_height,
        }
    }
}

/// Scale a value using sqrt.
///
/// If the passed value is negative, it is scaled as if it were a
/// positive value. Ex: `sign_safe_sqrt(-4)` returns `-2`.
///
/// # Arguments
/// * `x` - Value to scale
///
/// # Return
/// Sqrt-scaled value.
fn sign_safe_sqrt(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    } else if x > 0.0 {
        return x.sqrt();
    } else {
        return -((-x).sqrt());
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use cgmath::Vector3;

    use tectonic::*;

    #[test]
    fn test_plate_motion_differs() {
        let mut tectonic = TectonicLayer::new(1);
        let motion1 = tectonic.plate(
            Vector3::new(1, 2, -3), Vector3::new(1.0, 2.0, -3.0)
        ).unwrap().motion;
        let motion2 = tectonic.plate(
            Vector3::new(1, 2, 3), Vector3::new(1.0, 2.0, 3.0)
        ).unwrap().motion;
        let motion3 = tectonic.plate(
            Vector3::new(-1, -2, -3), Vector3::new(-1.0, -2.0, 3.0)
        ).unwrap().motion;

        assert_ne!(motion1, motion2);
        assert_ne!(motion1, motion3);
    }

    #[test]
    fn test_plate_motion_is_consistent() {
        let mut tectonic = TectonicLayer::new(1);
        let motion1 = tectonic.plate(
            Vector3::new(1, 2, 3), Vector3::new(1.0, 2.0, 3.0)
        ).unwrap().motion;
        let motion2 = tectonic.plate(
            Vector3::new(1, 2, 3), Vector3::new(1.0, 2.0, 3.0)
        ).unwrap().motion;

        assert_eq!(motion1, motion2);
    }
}
