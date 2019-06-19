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

use std::num::Wrapping;

use cgmath::{Vector3, Vector4};
use cgmath::MetricSpace;
use cgmath::InnerSpace;
use num_traits::real::Real;


// --------------------------------------------------------------------
// Structs


pub struct VoronoiSpace {
    pub region_shape: Vec3,
    pub seed: u32,
    pub nuclei_per_region: u8,
}

pub struct Cell {
    pub nucleus: Vec3,
    pub indices: Vector4<i64>,
    pub neighbors: Vec<Neighbor>
}

#[derive(Clone)]
pub struct Neighbor {
    pub nucleus: Vec3,
    pub indices: Vector4<i64>,
    pub rel_pos: Vec3,
    pub distance: f64
}


// --------------------------------------------------------------------
// Implementations


impl VoronoiSpace {
    pub const DEFAULT_NUCLEI_PER_REGION: u8 = 4;

    // Constructor

    pub fn new(seed: u32, region_shape: Vec3) -> Self {
        VoronoiSpace {
            seed,
            region_shape,
            nuclei_per_region: Self::DEFAULT_NUCLEI_PER_REGION,
        }
    }

    /// Get cell which contains a position.
    pub fn cell(&self, v: Vec3) -> Cell {
        let region_indices: Vector3<i64> = self.region(v);
        let region_nuclei: Vec<Vec3> = self.region_points(region_indices);

        // Find center.
        let mut nucleus: Vec3 = Vec3::new(0.0, 0.0, 0.0);
        let mut min_d: f64 = -1.0;
        let mut nucleus_indices: Vector4<i64> = Vector4::new(0, 0, 0, 0);
        self.visit_cluster(region_indices, &mut |nucleus2, indices2| {
            let d = nucleus.distance2(v);
            if min_d < 0.0 || d < min_d {
                min_d = d;
                nucleus = nucleus2;
                nucleus_indices = indices2;
            }
        });

        assert!(min_d >= 0.0);

        // Find nearby cells.
        let mut nearby_cells: Vec<Neighbor> = Vec::with_capacity(
            (self.nuclei_per_region as usize) * 5 * 5 * 5);
        let nucleus_region = Vector3::new(
                nucleus_indices.x, nucleus_indices.y, nucleus_indices.z
        );
        self.visit_super_cluster(nucleus_region, &mut |nucleus2, indices2 | {
            // Skip the nucleus that is this cell's nucleus.
            if indices2 == nucleus_indices {
                return;
            }

            let rel_pos = nucleus2 - nucleus;
            let distance = nucleus.distance2(nucleus2);

            nearby_cells.push(Neighbor {
                nucleus: nucleus2,
                indices: indices2,
                rel_pos,
                distance
            })
        });
        nearby_cells.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        // Find neighbors from nearby cells.
        let mut neighbors: Vec<Neighbor> = Vec::new();
        for nearby in nearby_cells {
            let mut excluded = false;
            for neighbor in &neighbors {
                let near_dir = nearby.rel_pos.normalize();
                let neighbor_dir = neighbor.rel_pos.normalize();
                let angle_cos = near_dir.dot(neighbor_dir);

                // If cos of angle < 0 then angle is > 90 degrees.
                if angle_cos < 0.0 {
                    continue;
                }

                if nearby.distance * angle_cos > neighbor.distance {
                    excluded = true;
                    break;
                }
            }

            if !excluded {
                neighbors.push(nearby);
            }
        }

        // Create cell
        Cell {
            nucleus,
            indices: nucleus_indices,
            neighbors
        }
    }

    /// Visit all nuclei in the identified region and all regions
    /// adjacent to it.
    ///
    /// The visited regions form a 3x3x3 cube.
    pub fn visit_cluster(
            &self,
            center_indices: Vector3<i64>,
            f: &mut FnMut(Vec3, Vector4<i64>)
    ) {
        for i in -1..2 {
            for j in -1..2 {
                for k in -1..2 {
                    let region_points = self.region_points(Vector3::new(
                        center_indices.x + i,
                        center_indices.y + j,
                        center_indices.z + k
                    ));
                    for m in 0..self.nuclei_per_region {
                        let nucleus: Vec3 = region_points[m as usize];
                        let indices = Vector4::new(i, j, k, m as i64);
                        f(nucleus, indices);
                    }
                }
            }
        }
    }

    /// Visit all nuclei in the identified region and all regions
    /// adjacent to it.
    ///
    /// The visited regions form a 5x5x5 cube, excepting the outermost
    /// corners (-2, -2, -2), (2, 2, 2), etc
    pub fn visit_super_cluster(
        &self,
        center_indices: Vector3<i64>,
        f: &mut FnMut(Vec3, Vector4<i64>)
    ) {
        for i in -2i64..3 {
            for j in -2i64..3 {
                for k in -2i64..3 {
                    // Skip box corners
                    if k.abs() == 2 && i.abs() == 2 && j.abs() == 2 {
                        continue;
                    }

                    let region_points = self.region_points(Vector3::new(
                        center_indices.x + i,
                        center_indices.y + j,
                        center_indices.z + k
                    ));
                    for m in 0..self.nuclei_per_region {
                        let nucleus: Vec3 = region_points[m as usize];
                        let indices = Vector4::new(i, j, k, m as i64);
                        f(nucleus, indices);
                    }
                }
            }
        }
    }

    // Helper functions

    /// Get region which contains passed position vector
    ///
    /// Returns Vector3 of the regions x, y, z indices.
    fn region(&self, v: Vec3) -> cgmath::Vector3<i64> {
        Vector3::new(
            (v.x / self.region_shape.x).floor() as i64,
            (v.y / self.region_shape.y).floor() as i64,
            (v.z / self.region_shape.z).floor() as i64
        )
    }

    /// Gets position of each point in a region.
    fn region_points(&self, region: Vector3<i64>) -> Vec<Vec3> {
        let mut vec = Vec::with_capacity(self.nuclei_per_region as usize);

        for i in 0..self.nuclei_per_region {
            vec.push(self.region_point(region, i));
        }

        return vec
    }

    /// Gets position of region point of index i in world space.
    fn region_point(&self, region: Vector3<i64>, i: u8) -> Vec3 {
        let seed_hash = Wrapping(idx_hash(self.seed as i64));
        let i_hash = Wrapping(idx_hash(i as i64));
        let x_hash = Wrapping(idx_hash(region.x));
        let y_hash = Wrapping(idx_hash(region.y));
        let z_hash = Wrapping(idx_hash(region.z));
        let hash: u32 = (seed_hash + i_hash + x_hash + y_hash + z_hash).0;

        let region_pos = component_multiply(self.region_shape, rand3(hash));

        return self.region_origin(region) + region_pos;
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

    // Getters + Setters

    pub fn set_nuclei_per_region(self, n: u8) -> Self {
        Self {
                nuclei_per_region: n,
                ..self
        }
    }

    pub fn nuclei_per_region(self) -> u8 {
        self.nuclei_per_region
    }
}


// --------------------------------------------------------------------


impl Cell {
    fn contains(&self, position: Vec3) -> bool {
        let rel_pos = position - self.nucleus;
        let rel_mag2 = rel_pos.magnitude() * 2.0;
        let rel_dir = rel_pos.normalize();
        let mut contains = true;
        for neighbor in &self.neighbors {
            let neighbor_dir = neighbor.rel_pos.normalize();
            let angle_cos = neighbor_dir.dot(rel_dir);
            if angle_cos < 0.0 {
                continue;
            }
            if rel_mag2 * angle_cos > neighbor.distance {
                contains = false;
                break;
            }
        }
        return contains;
    }
}


// --------------------------------------------------------------------


///
fn idx_hash(x: i64) -> u32 {
    let x = Wrapping(x as u32);

    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    let x = (x >> 16) ^ x;

    return x.0;
}


/// Produces a Vec3 from a random u32.
///
/// Produced x, y, and z values are all between 0 and 1.
fn rand3(x: u32) -> Vec3 {
    Vec3::new(
        ((x & 0x7FF) as f64) / 2048.0,
        ((x & (0x3FF << 11)) as f64) / 1024.0,
        ((x & (0x7FF << 21)) as f64) / 2048.0
    )
}


/// Multiply vectors component-wise.
fn component_multiply(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(
        a.x * b.x,
        a.y * b.y,
        a.z * b.z
    )
}


#[cfg(test)]
mod tests {

    use voronoi::*;
    use cgmath::Vector3;

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
    fn test_component_wise_vector_multiplication() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(2.0, 3.0, 4.0);

        let r = component_multiply(a, b);

        assert_eq!(r.x, 2.0);
        assert_eq!(r.y, 6.0);
        assert_eq!(r.z, 12.0);
    }

    #[test]
    fn test_region_has_consistent_points() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let points1 = voronoi.region_points(Vector3::new(1, 2, 3));
        let points2 = voronoi.region_points(Vector3::new(1, 2, 3));

        assert_eq!(
            VoronoiSpace::DEFAULT_NUCLEI_PER_REGION as usize,
            points1.len()
        );

        assert_eq!(points1[0], points2[0]);
        assert_eq!(points1[1], points2[1]);
        assert_eq!(points1[2], points2[2]);
        assert_eq!(points1[3], points2[3]);
    }

    #[test]
    fn test_points_differ_within_regions() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let points = voronoi.region_points(Vector3::new(1, 2, 3));

        assert_eq!(
            VoronoiSpace::DEFAULT_NUCLEI_PER_REGION as usize,
            points.len()
        );

        assert_ne!(points[0], points[1]);
        assert_ne!(points[0], points[2]);
        assert_ne!(points[0], points[3]);
        assert_ne!(points[1], points[2]);
        assert_ne!(points[1], points[3]);
        assert_ne!(points[2], points[3]);
    }

    #[test]
    fn test_points_differ_between_regions() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let points1 = voronoi.region_points(Vector3::new(1, 2, 3));
        let points2 = voronoi.region_points(Vector3::new(3, 5, 7));

        assert_eq!(
            VoronoiSpace::DEFAULT_NUCLEI_PER_REGION as usize,
            points1.len()
        );

        assert_ne!(points1[0], points2[0]);
        assert_ne!(points1[1], points2[1]);
        assert_ne!(points1[2], points2[2]);
        assert_ne!(points1[3], points2[3]);
    }

    #[test]
    fn test_points_differ_between_inverse_regions() {
        let voronoi = VoronoiSpace::new(0, Vec3::new(10.0, 10.0, 10.0));

        let points1 = voronoi.region_points(Vector3::new(0, 0, 0));
        let points2 = voronoi.region_points(Vector3::new(-1, -1, -1));

        assert_eq!(
            VoronoiSpace::DEFAULT_NUCLEI_PER_REGION as usize,
            points1.len()
        );

        assert_ne!(points1[0], points2[0]);
        assert_ne!(points1[1], points2[1]);
        assert_ne!(points1[2], points2[2]);
        assert_ne!(points1[3], points2[3]);
    }


    // ----------------------------------------------------------------


    #[test]
    fn test_cell_contains() {
        let mut neighbors: Vec<Neighbor> = Vec::with_capacity(5);

        neighbors.push(Neighbor {
            nucleus: Vec3::new(0.0, 3.0, 2.0),
            indices: Vector4::new(0, 0, 0, 0),  // not used.
            rel_pos: Vec3::new(0.0, 2.0, 0.0),
            distance: 2.0
        });
        neighbors.push(Neighbor {
            nucleus: Vec3::new(1.0, 1.0, 2.0),
            indices: Vector4::new(0, 0, 0, 0),  // not used.
            rel_pos: Vec3::new(2.0, 0.0, 0.0),
            distance: 1.0
        });
        neighbors.push(Neighbor {
            nucleus: Vec3::new(-1.5, 3.0, 2.0),
            indices: Vector4::new(0, 0, 0, 0),  // not used.
            rel_pos: Vec3::new(-1.5, 0.0, 0.0),
            distance: 1.5
        });
        neighbors.push(Neighbor {
            nucleus: Vec3::new(0.0, -3.0, 2.0),
            indices: Vector4::new(0, 0, 0, 0),  // not used.
            rel_pos: Vec3::new(0.0, -4.0, 0.0),
            distance: 4.0
        });
        neighbors.push(Neighbor {
            nucleus: Vec3::new(0.0, 0.0, 3.0),
            indices: Vector4::new(0, 0, 0, 0),  // not used.
            rel_pos: Vec3::new(0.0, 0.0, 1.0),
            distance: 1.0
        });

        let cell = Cell {
            nucleus: Vec3::new(0.0, 1.0, 2.0),
            indices: Vector4::new(0, 0, 0, 0),  // Not used.
            neighbors
        };

        assert!(cell.contains(Vec3::new(0.0, 1.0, 2.0)));
        assert!(cell.contains(Vec3::new(0.0, -0.5, 2.25)));
        assert!(!cell.contains(Vec3::new(0.0, -2.5, 2.25)));
        assert!(!cell.contains(Vec3::new(0.0, 1.0, 2.6)));
    }
}
