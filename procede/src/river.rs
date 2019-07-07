//! Module containing river procedural generation structs
//! and functions.
use std::f64;

use aabb_quadtree::{QuadTree, ItemId};
use aabb_quadtree::geom::{Rect, Point};
use cgmath::{Vector2, Vector3};
use cgmath::InnerSpace;
use lru_cache::LruCache;
use std::collections::{VecDeque, HashSet, HashMap};
use std::usize;

use tectonic::{TectonicLayer, TectonicInfo};
use util::{hash_indices, sphere_uv_vec};


// --------------------------------------------------------------------


pub struct RiverLayer {
    seed: u32,
    region_cache: LruCache<Vector3<i64>, Region>,
}

/// Struct used to return height and related information about
/// a position from the RiverLayer.
pub struct RiverInfo {
    height: f64,
}

/// A River Region is associated with a single tectonic cell and
struct Region {
    segment_tree: QuadTree<Segment>,
    nodes: Vec<Node>,
}

/// River node
struct Node {
    i: usize,  // Index of Node in river graph.
    indices: Vector2<i64>,
    h: f64,  // Height above mean sea level.
    neighbors: [usize; 3],  // Neighboring nodes within graph.
    inlets: [usize; 2],
    outlet: usize,
    strahler: i16
}

struct GenerationInfo {
    mouths: Vec<usize>,
    low_corner: Vector2<f64>,
    high_corner: Vector2<f64>,
}

/// River segment
struct Segment {
    a: Vector2<f64>,
    b: Vector2<f64>
}

struct HexGraph {
    edge_len: f64,  // Distance from vertex to vertex.
    seq_len: f64,  // Y-distance covered by 4 vertex sequence.
    x_step: f64,  // X-distance covered by a single x increment.
}

// --------------------------------------------------------------------


impl RiverLayer {
    pub const REGION_CACHE_SIZE: usize = 1_00;

    // Construction

    fn new(
            seed: u32,
            tectonic: &mut TectonicLayer,
            tectonic_info: TectonicInfo
    ) -> RiverLayer {
        RiverLayer {
            seed,
            region_cache: LruCache::new(Self::REGION_CACHE_SIZE),
        }
    }

    // Height methods

    /// Produces height and related information for a position.
    ///
    /// # Arguments
    /// * `v` - Position in 3d space relative to world center.
    ///             Will be normalized.
    /// * `tectonic_info` - Tectonic information for the passed point
    /// * `tectonic` - Mutable reference to tectonic layer.
    ///
    /// # Returns
    /// RiverInfo containing height and related information.
    pub fn height(
            &mut self,
            v: Vector3<f64>,
            tectonic_info: TectonicInfo,
            tectonic: &mut TectonicLayer,
    ) -> RiverInfo {
        let indices: Vector3<i64> = tectonic_info.indices;
        if !self.region_cache.contains_key(&indices) {
            let region_hash = hash_indices(self.seed, indices);
            let region = Region::new(region_hash, tectonic, tectonic_info);
            self.region_cache.insert(indices, region);
        }
        self.region_cache.get_mut(&indices).unwrap().height(v)
    }
}


// --------------------------------------------------------------------


/// River region.
///
/// A River region is associated with a single tectonic cell, and
/// handles height generation due to river action within its bounds.
///
/// A Tectonic cell is an ideal boundary for a river region because it
/// is likely to either border an ocean, or else be bordered by a
/// mountain range which would realistically separate river basins.
impl Region {
    pub const NODE_MEAN_SEPARATION: f64 = 10_000.0;

    fn new(
        seed: u32,
        tectonic: &mut TectonicLayer,
        tectonic_info: TectonicInfo,
    ) -> Region {
        // Get nucleus surface position mapping
        let center3d = tectonic.surface.surf_pos(
            tectonic_info.nucleus.normalize()
        );
        let (u_vec, v_vec) = sphere_uv_vec(center3d);

        let mut nodes = Self::create_nodes(seed, tectonic, tectonic_info);

        // Connect nodes in-place.
        let info = Self::generate_rivers(&mut nodes);

        // Create bounding shape
        let shape = Rect{
            top_left: vec2pt(info.low_corner),
            bottom_right: vec2pt(info.high_corner),
        };

        // Create river segments
        let segment_tree = Self::create_segments(&nodes, &info.mouths, shape);

        Region {
            segment_tree,
            nodes,
        }
    }

    /// Creates nodes that lie within a river region.
    ///
    /// # Arguments
    /// * `seed` - Seed for node graph.
    /// * `tectonic` - Mutable reference to TectonicLayer.
    /// * `tectonic_info` - used to indicate the region.
    ///
    /// # Return
    /// Vec of nodes with all fields except inlets, outlet, and
    /// strahler set.
    fn create_nodes(
        seed: u32,
        tectonic: &mut TectonicLayer,
        tectonic_info: TectonicInfo,
    ) -> Vec<Node> {
        // Find the first hex index contained within the cell.
        fn find_first(
            tectonic: &mut TectonicLayer,
            cell_indices: Vector3<i64>,
            hex_graph: &HexGraph
        ) -> Vector2<i64> {
            // TODO: Ensure first node is in cell. If not, do quick search.
            Vector2::new(0, 0)
        }

        // Run BFS Search until all nodes that are in cell are added
        // to nodes Vec.
        // Included indices are added to included set for
        // quick checking.
        fn explore_cell(
            tectonic: &mut TectonicLayer,
            cell_indices: Vector3<i64>,
            hex_graph: &HexGraph,
            first: Vector2<i64>,
        ) -> (Vec<Node>, HashMap<Vector2<i64>, usize>) {
            let mut nodes = Vec::new();
            let mut included = HashMap::with_capacity(100);
            let mut frontier = VecDeque::with_capacity(100);
            let mut visited = HashSet::with_capacity(100);
            frontier.push_back(first);
            visited.insert(first);
            while !frontier.is_empty() {
                let indices = frontier.pop_front().unwrap();
                let uv = hex_graph.pos(indices);  // TODO: randomize
                let xyz = Region::uv_to_xyz_norm(uv);
                let node_info = tectonic.height(xyz);
                if node_info.indices != cell_indices {
                    continue;
                }

                // Add indices to included map and append node to vec.
                included.insert(indices, nodes.len());
                let node_i = nodes.len();
                nodes.push(Node {
                    i: node_i,
                    indices,
                    h: node_info.height,
                    neighbors: [usize::MAX, usize::MAX, usize::MAX],
                    inlets: [usize::MAX, usize::MAX],
                    outlet: usize::MAX,
                    strahler: -1
                });

                // Add hex neighbors to frontier.
                for hex_neighbor in &hex_graph.neighbors(indices) {
                    if !visited.contains(&hex_neighbor) {
                        frontier.push_back(*hex_neighbor);
                    }
                }
            }

            (nodes, included)
        }

        // Set node neighbors.
        // These are the nodes which are within the cell. If a cell has
        // fewer than three neighbors, one or more index will be set to
        // -1 (since usize is unsigned, this will be usize::MAX)
        fn set_neighbors(
                nodes: &mut Vec<Node>,
                included: HashMap<Vector2<i64>, usize>,
                hex_graph: &HexGraph,
        ) {
            for node in nodes {
                let hex_neighbors = hex_graph.neighbors(node.indices);
                for (i, neighbor_indices) in hex_neighbors.iter().enumerate() {
                    // `included` contains node index stored by hex indices key.
                    let node_index = included.get(neighbor_indices);
                    if node_index.is_some() {
                        node.neighbors[i] = *node_index.unwrap();
                    }
                }
            }
        }

        // Create hex graph to produce nodes.
        let hex_graph = HexGraph::new(Self::NODE_MEAN_SEPARATION);

        // Find center node.
        let first = find_first(tectonic, tectonic_info.indices, &hex_graph);

        // Find nodes in cell.
        let (mut nodes, included) = explore_cell(
            tectonic,
            tectonic_info.indices,
            &hex_graph,
            first,
        );
        set_neighbors(&mut nodes, included, &hex_graph);

        nodes
    }

    /// Connects nodes in-place to form rivers.
    ///
    /// # Arguments
    /// * `nodes` River Nodes. This vector will be modified in-place.
    ///
    /// # Return
    /// GenerationInfo with Vec of river mouth nodes and other info.
    fn generate_rivers(nodes: &mut Vec<Node>) -> GenerationInfo {
        // Todo: Find river mouths
        // Todo: Form Tree using randomized search
        GenerationInfo {
            mouths: Vec::default(),
            low_corner: Vector2::new(0.0, 0.0),
            high_corner: Vector2::new(0.0, 0.0),
        }
    }

    /// Finds nodes that represent river mouths.
    ///
    /// # Arguments
    /// * `nodes` Reference to vector of nodes which will be searched.
    ///
    /// # Return
    /// Vector of indices of nodes which are river mouths.
    fn river_mouths(nodes: &Vec<Node>) -> Vec<i32> {
        Vec::default()  // Todo: Search
    }

    /// Create river segments from nodes.
    ///
    /// # Arguments
    /// * `nodes` - Reference to Vec of river nodes.
    /// * `mouths` - Reference to Vec of indices indicating the nodes
    ///             at which rivers begin.
    /// * `shape` - Reference to river node bounds.
    ///
    /// # Return
    /// Searchable QuadTree of river segments.
    fn create_segments(
        nodes: &Vec<Node>,
        mouths: &Vec<usize>,
        shape: Rect,
    ) -> QuadTree<Segment> {
        QuadTree::default(shape) // Todo
    }

    // --------------

    /// Gets height at passed position
    ///
    /// # Arguments
    /// * `v` - Position relative to world center.
    fn height(&self, v: Vector3<f64>) -> RiverInfo {
        let uv = self.xyz_to_uv(v);
        let (d, nearest_seg) = self.nearest_segment(uv);

        RiverInfo {
            height: 0.0, // TODO: Replace placeholder
        }
    }

    /// Converts a 3d position vector to a 2d uv position.
    ///
    /// The produced vector can be used to identify a position
    /// in the 2d river graph.
    ///
    /// # Arguments
    /// * `v` - Position relative to world center.
    ///
    /// # Return
    /// 2d UV position in plane tangential to region origin.
    fn xyz_to_uv(&self, v: Vector3<f64>) -> Vector2<f64> {
        Vector2::new(0.0, 0.0)  // Todo: Replace placeholder
    }

    /// Converts a uv position vector to a 3d world position.
    ///
    /// The produced vector identifies a point in 3d space relative
    /// to the world center.
    ///
    /// # Arguments:
    /// * `uv` - 2d uv position vector.
    ///
    /// # Return
    /// Normalized vector identifying point on surface of world sphere.
    fn uv_to_xyz_norm(uv: Vector2<f64>) -> Vector3<f64> {
        Vector3::new(0.0, 0.0, 0.0)  // Todo: Replace placeholder
    }

    /// Finds the nearest river segment to a position.
    ///
    /// # Arguments:
    /// * `uv` - Position in uv space relative to Region center.
    ///
    /// # Returns
    /// * Distance to nearest segment.
    /// * Segment nearest the passed point.
    fn nearest_segment(&self, uv: Vector2<f64>) -> (f64, Segment) {
        (  // TODO: Get distance, nearest segment.
            -1.0,
            Segment {a: Vector2::new(-1.0, -1.0), b: Vector2::new(1.0, 1.0)}
        )
    }
}


// --------------------------------------------------------------------


impl HexGraph {
    /// Constructs new HexGraph
    fn new(edge_len: f64) -> HexGraph {
        HexGraph {
            edge_len,
            seq_len: edge_len * 3.0,
            x_step: edge_len * 2.0 * (f64::consts::PI / 3.0).sin(),
        }
    }

    /// Gets position of vertex with passed indices.
    fn pos(&self, indices: Vector2<i64>) -> Vector2<f64> {
        // Get index within 4 vertex sequence.
        // This statement is a workaround for the '%' operator
        // producing the remainder, rather than the modulo.
        let i = ((indices.y % 4) + 4) % 4;

        let seq_iteration;
        if indices.y >= 0 {
            seq_iteration = indices.y / 4;
        } else {
            seq_iteration = indices.y / 4 - 1;
        }

        let seq_pos0 = Vector2::new(
            indices.x as f64 * self.x_step,
            seq_iteration as f64 * self.seq_len
        );

        // Find pos
        match i {
            0 => seq_pos0,
            1 => Vector2::new(seq_pos0.x, seq_pos0.y + self.edge_len),
            2 => Vector2::new(
                seq_pos0.x + (f64::consts::PI / 3.0).sin() * self.edge_len,
                seq_pos0.y + self.edge_len * 1.5
            ),
            3 => Vector2::new(
                seq_pos0.x + (f64::consts::PI / 3.0).sin() * self.edge_len,
                seq_pos0.y + self.edge_len * 2.5
            ),
            _ => panic!("Unexpected sequence index: {}", i)
        }
    }

    /// Gets indices of neighbors sharing an edge with a vertex.
    fn neighbors(&self, indices: Vector2<i64>) -> [Vector2<i64>; 3] {
        // Get index within 4 vertex sequence.
        // This statement is a workaround for the '%' operator
        // producing the remainder, rather than the modulo.
        let i = ((indices.y % 4) + 4) % 4;

        match i {
            0 => [
                Vector2::new(indices.x, indices.y + 1),
                Vector2::new(indices.x, indices.y - 1),
                Vector2::new(indices.x - 1, indices.y - 1)
            ],
            1 => [
                Vector2::new(indices.x, indices.y + 1),
                Vector2::new(indices.x, indices.y - 1),
                Vector2::new(indices.x - 1, indices.y + 1)
            ],
            2 => [
                Vector2::new(indices.x, indices.y + 1),
                Vector2::new(indices.x + 1, indices.y - 1),
                Vector2::new(indices.x, indices.y - 1)
            ],
            3 => [
                Vector2::new(indices.x, indices.y + 1),
                Vector2::new(indices.x + 1, indices.y + 1),
                Vector2::new(indices.x, indices.y - 1)
            ],
            _ => panic!("Unexpected sequence index: {}", i)
        }
    }
}


/// Converts Vector2 to Point for use in QuadTree.
///
/// As the precision is lowered from f64 to f32, some information
/// will be lost in the conversion.
///
/// # Arguments
/// * `v` - Vector2 to be converted to a Point.
///
/// # Return
/// Point
#[inline]
fn vec2pt(v: Vector2<f64>) -> Point {
    Point {
        x: v.x as f32,
        y: v.y as f32,
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use cgmath::Vector2;
    use cgmath::MetricSpace;

    use river::HexGraph;


    #[test]
    fn test_graph_vertex_pos() {
        let graph = HexGraph::new(1.0);

        let p00 = graph.pos(Vector2::new(0, 0));
        let p01 = graph.pos(Vector2::new(0, 1));
        let p02 = graph.pos(Vector2::new(0, 2));
        let p03 = graph.pos(Vector2::new(0, 3));
        let p04 = graph.pos(Vector2::new(0, 4));
        let p0n1 = graph.pos(Vector2::new(0, -1));
        let pn12 = graph.pos(Vector2::new(-1, 2));
        let p12 = graph.pos(Vector2::new(1, 2));
        let p10 = graph.pos(Vector2::new(1, 0));

        assert_vec2_near!(p00, Vector2::new(0.0, 0.0));
        assert_vec2_near!(p01, Vector2::new(0.0, 1.0));
        assert_vec2_near!(p02, Vector2::new(0.866025403, 1.5));
        assert_vec2_near!(p03, Vector2::new(0.866025403, 2.5));
        assert_vec2_near!(p04, Vector2::new(0.0, 3.0));
        assert_vec2_near!(p0n1, Vector2::new(0.866025403, -0.5));
        assert_vec2_near!(pn12, Vector2::new(-0.866025403, 1.5));
        assert_vec2_near!(p12, Vector2::new(2.598076211353316, 1.5));
        assert_vec2_near!(p10, Vector2::new(1.7320508, 0.0));
    }

    #[test]
    fn test_graph_neighbors_of_i0() {
        let graph = HexGraph::new(1.0);

        let neighbors = graph.neighbors(Vector2::new(0, 0));
        assert_eq!(neighbors[0], Vector2::new(0, 1));
        assert_eq!(neighbors[1], Vector2::new(0, -1));
        assert_eq!(neighbors[2], Vector2::new(-1, -1));
    }

    #[test]
    fn test_graph_neighbors_of_i1() {
        let graph = HexGraph::new(1.0);

        let neighbors = graph.neighbors(Vector2::new(1, 1));
        assert_eq!(neighbors[0], Vector2::new(1, 2));
        assert_eq!(neighbors[1], Vector2::new(1, 0));
        assert_eq!(neighbors[2], Vector2::new(0, 2));
    }

    #[test]
    fn test_graph_neighbors_of_i2() {
        let graph = HexGraph::new(1.0);

        let neighbors = graph.neighbors(Vector2::new(-1, 2));
        assert_eq!(neighbors[0], Vector2::new(-1, 3));
        assert_eq!(neighbors[1], Vector2::new(0, 1));
        assert_eq!(neighbors[2], Vector2::new(-1, 1));
    }

    #[test]
    fn test_graph_neighbors_of_i3() {
        let graph = HexGraph::new(1.0);

        let neighbors = graph.neighbors(Vector2::new(0, -1));
        assert_eq!(neighbors[0], Vector2::new(0, 0));
        assert_eq!(neighbors[1], Vector2::new(1, 0));
        assert_eq!(neighbors[2], Vector2::new(0, -2));
    }

    #[test]
    fn test_neighbor_distances() {
        let test_indices = [
            Vector2::new(0, 0),
            Vector2::new(1, 1),
            Vector2::new(-1, 2),
            Vector2::new(0, -1)
        ];

        let graph = HexGraph::new(1.0);
        for vertex in &test_indices {
            let neighbors = graph.neighbors(*vertex);

            for neighbor in &neighbors {
                let pos0 = graph.pos(*vertex);
                let pos1 = graph.pos(*neighbor);

                assert_approx_eq!(pos0.distance2(pos1), 1.0, 1e-6);
            }
        }
    }
}
