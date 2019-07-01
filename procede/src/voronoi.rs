/// # Voronoi
///
/// Specialized voronoi implementation and related helper
/// functions + types.
///
/// ## Terminology:
///     Nuclei: a Cell center.
///     Cell: the area of space around a nucleus which is closer to
///             that nucleus than any other.
///     Region: An area of space within which N nuclei are generated.
///     Cluster: A region along with all its adjacent regions.
///

type Vec3 = cgmath::Vector3<f64>;

use cgmath::Vector3;
use cgmath::MetricSpace;

use util::{rand3, component_multiply, hash_indices};


// --------------------------------------------------------------------
// Structs


pub struct VoronoiSpace {
    pub region_shape: Vec3,
    pub seed: u32,
}

pub struct NearResult {
    pub regions: [Vector3<i64>; 4],
    pub points: [Vec3; 4],
    pub dist: [f64; 4]
}


// --------------------------------------------------------------------
// Implementations


impl VoronoiSpace {
    pub fn new(seed: u32, region_shape: Vec3) -> Self {
        VoronoiSpace {
            seed,
            region_shape
        }
    }

    /// Find nearest points
    pub fn near4(&self, v: Vec3) -> NearResult {
        let v_region = self.region(v);

        let mut result: NearResult = NearResult {
            regions: [Vector3::new(0, 0, 0); 4],
            points: [Vec3::new(0.0, 0.0, 0.0); 4],
            dist: [-1.0; 4]
        };

        for i in -2i64..3 {
            for j in -2i64..3 {
                for k in -2i64..3 {
                    // Skip box corners
                    if k.abs() == 2 && i.abs() == 2 && j.abs() == 2 {
                        continue;
                    }

                    // Get indices of visted region
                    let indices = Vector3::new(
                        v_region.x + i,
                        v_region.y + j,
                        v_region.z + k
                    );

                    // Get point
                    let p = self.region_point(indices);
                    let d2 = p.distance2(v);

                    // Find place to store point
                    let mut place = 4;
                    while place > 0 {
                        if result.dist[place - 1] < 0.0 ||
                                d2 < result.dist[place - 1] {
                            place -= 1;
                        } else {
                            break;
                        }
                    }

                    // Place point in result and shift others.
                    let mut t1_i = indices;
                    let mut t1_p = p;
                    let mut t1_d = d2;
                    while place < 4 {
                        let t2_i = result.regions[place];
                        let t2_p = result.points[place];
                        let t2_d = result.dist[place];
                        result.regions[place] = t1_i;
                        result.points[place] = t1_p;
                        result.dist[place] = t1_d;
                        t1_i = t2_i;
                        t1_p = t2_p;
                        t1_d = t2_d;

                        place += 1;
                    }
                }
            }
        }

        result.dist[0] = result.dist[0].sqrt();
        result.dist[1] = result.dist[1].sqrt();
        result.dist[2] = result.dist[2].sqrt();
        result.dist[3] = result.dist[3].sqrt();

        result
    }

    // Helper functions

    /// Get region which contains passed position vector
    ///
    /// Returns Vector3 of the regions x, y, z indices.
    pub fn region(&self, v: Vec3) -> cgmath::Vector3<i64> {
        Vector3::new(
            (v.x / self.region_shape.x).floor() as i64,
            (v.y / self.region_shape.y).floor() as i64,
            (v.z / self.region_shape.z).floor() as i64
        )
    }

    /// Gets position of region point of index i in world space.
    fn region_point(&self, indices: Vector3<i64>) -> Vec3 {
        let hash: u32 = hash_indices(self.seed, indices);
        let region_pos = component_multiply(self.region_shape, rand3(hash));
        return self.region_origin(indices) + region_pos;
    }

    /// Gets the origin point of a region.
    ///
    /// This is the position in the 'lower' x, y, and z dimensions.
    fn region_origin(&self, region: Vector3<i64>) -> Vec3 {
        return Vec3::new(
            self.region_shape.x * region.x as f64,
            self.region_shape.y * region.y as f64,
            self.region_shape.z * region.z as f64
        )
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {

    use cgmath::Vector3;

    use voronoi::*;

    #[test]
    fn test_get_region() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        assert_eq!(
            voronoi.region(Vec3::new(0.5, 9.0, 3.0)),
            Vector3::new(0, 0, 0)
        );

        assert_eq!(
            voronoi.region(Vec3::new(15.1, 9.0, 3.0)),
            Vector3::new(1, 0, 0)
        );

        assert_eq!(
            voronoi.region(Vec3::new(15.1, -5.0, -3.0)),
            Vector3::new(1, -1, -1)
        );
    }

    #[test]
    fn test_region_has_consistent_points() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let point1 = voronoi.region_point(Vector3::new(1, 2, 3));
        let point2 = voronoi.region_point(Vector3::new(1, 2, 3));

        assert_eq!(point1, point2);
    }

    #[test]
    fn test_points_differ_between_regions() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let point1 = voronoi.region_point(Vector3::new(1, 2, 3));
        let point2 = voronoi.region_point(Vector3::new(3, 5, 7));

        assert_ne!(point1, point2);
    }

    #[test]
    fn test_points_differ_between_inverse_regions() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let point1 = voronoi.region_point(Vector3::new(0, 0, 0));
        let point2 = voronoi.region_point(Vector3::new(-1, -1, -1));

        assert_ne!(point1, point2);
    }
}
