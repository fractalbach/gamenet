//! Module containing river procedural generation structs
//! and functions.
use std::f64;
use std::num::Wrapping;

use aabb_quadtree::{QuadTree, ItemId};
use aabb_quadtree::geom::{Rect, Point};
use cgmath::{Vector2, Vector3};
use cgmath::InnerSpace;
use lru_cache::LruCache;
use std::cmp::Ordering;
use std::collections::{VecDeque, HashSet, HashMap, BinaryHeap};
use std::usize;

use tectonic::{TectonicLayer, TectonicInfo};
use util::{hash_indices, sphere_uv_vec, idx_hash};


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
    uv: Vector2<f64>,
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
        /// Find the first hex index contained within the cell.
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
                    uv,
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

        // ------------------------

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
    /// This method builds rivers 'backwards', starting at river mouth
    /// nodes, and proceeding upwards in elevation until all nodes that
    /// can be reached have been.
    ///
    /// # Arguments
    /// * `nodes` - River Nodes. This vector will be modified in-place.
    ///
    /// # Return
    /// GenerationInfo with Vec of river mouth nodes and other info.
    fn generate_rivers(nodes: &mut Vec<Node>) -> GenerationInfo {
        /// Struct representing a planned search along a single edge.
        ///
        /// This search will be carried out at some point
        /// in the future determined by a priority generated semi-
        /// randomly from its destination and origin
        ///
        /// An Expedition is assigned a semi-random priority based on
        /// its destination -and- origin, in order to avoid a pure
        /// breadth first search.
        #[derive(Copy, Clone, Eq, PartialEq)]
        struct Expedition {
            priority: u32,
            destination: usize,
            origin: usize,
        }

        impl Expedition {
            /// Creates a new Expedition which represents a search
            /// along an edge from an origin node to a destination
            /// node.
            ///
            /// # Arguments
            /// * `destination` - Index of node which the Expedition
            ///             will explore.
            ///
            /// # Return
            /// new Expedition
            fn new(destination: usize, origin: usize) -> Expedition {
                Expedition {
                    priority: Self::find_priority(destination, origin),
                    destination,
                    origin,
                }
            }

            /// Creates a start point for exploration.
            ///
            /// This is an 'Expedition' without an origin.
            ///
            /// # Arguments
            /// * `destination` - Index of node to start
            ///             exploration at.
            ///
            /// # Return
            /// new Expedition
            fn start_point(destination: usize) -> Expedition {
                Expedition {
                    priority: idx_hash(destination as i64),
                    destination,
                    origin: usize::MAX
                }
            }

            /// Finds priority of an Expedition.
            fn find_priority(dest: usize, origin: usize) -> u32 {
                let hash = Wrapping(idx_hash(dest as i64)) +
                    Wrapping(idx_hash(origin as i64));
                hash.0
            }
        }

        // The priority queue (BinaryHeap) depends on `Ord`.
        impl Ord for Expedition {
            /// Compares priority of two Expeditions.
            fn cmp(&self, other: &Expedition) -> Ordering {
                // In case of a tie, compare positions.
                // Required to make implementations of `PartialEq` and
                // `Ord` consistent.
                self.priority.cmp(&other.priority)
                    .then_with(|| self.destination.cmp(&other.destination))
                    .then_with(|| self.origin.cmp(&other.origin))
            }
        }

        impl PartialOrd for Expedition {
            fn partial_cmp(&self, other: &Expedition) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        /// Updates corners which track the bounding box that
        /// contains all nodes of the generated river network.
        ///
        /// Given a point in UV space, this function will modify the
        /// passed corners so that the bounding box described by them
        /// contains the passed point.
        ///
        /// This function modifies its passed bounding box points.
        /// It does not return anything.
        ///
        /// # Arguments
        /// * `uv` - Position in UV space that will be contained by the
        ///             bounding box.
        /// * `low_corner` - Mutable reference to the point in UV space
        ///             defining the low-x, low-y corner of the
        ///             bounding box.
        /// * `high_corner` - Mutable reference to the point in UV
        ///             space defining the high-x, high-y corner of the
        ///             bounding box.
        fn expand_bounds(
                uv: Vector2<f64>,
                low_corner: &mut Vector2<f64>,
                high_corner: &mut Vector2<f64>
        ) {
            // Update low_corner or high_corner if needed.
            if uv.x < low_corner.x {
                low_corner.x = uv.x;
            } else if uv.x > high_corner.x {
                high_corner.x = uv.x;
            }
            if uv.y < low_corner.y {
                low_corner.y = uv.y;
            } else if uv.y > high_corner.y {
                high_corner.y = uv.y;
            }
        }

        /// Creates expeditions to all valid unexplored points which
        /// may be reached from a node.
        ///
        /// This function modifies the passed expeditions heap; it does
        /// not return anything.
        ///
        /// # Arguments
        /// * `origin` Reference to the node from which expeditions
        ///             will start.
        /// * `nodes` - Vec of nodes in the graph.
        /// * `visited` - Set of node indices which have already been
        ///             visited. Expeditions will not be created to
        ///             these nodes.
        /// * `expeditions` - Heap of expeditions which newly created
        ///             expeditions will be added to.
        fn create_expeditions(
                origin: &Node,
                nodes: &Vec<Node>,
                visited: &HashSet<usize>,
                expeditions: &mut BinaryHeap<Expedition>
        ) {
            for neighbor in &origin.neighbors {
                // If already explored, continue.
                if visited.contains(neighbor) {
                    continue;
                }

                // If the neighbor has a lower height than the node
                // that was just arrived at, then don't try to explore
                // it from here. Rivers can't flow uphill.
                if nodes[*neighbor].h < origin.h {
                    continue;
                }

                expeditions.push(Expedition::new(*neighbor, origin.i));
            }
        }

        // ------------------------

        let mouths = Self::find_mouths(nodes);

        let mut expeditions = BinaryHeap::with_capacity(100);
        let mut visited = HashSet::with_capacity(100);

        let mut low_corner = Vector2::new(0.0, 0.0);
        let mut high_corner = Vector2::new(0.0, 0.0);

        // Initialize expeditions with river mouths
        for mouth in &mouths {
            expeditions.push(Expedition::start_point(*mouth));
        }

        // Explore
        while !expeditions.is_empty() {
            let exp = expeditions.pop().unwrap();

            // If already explored, continue.
            // Although expeditions are only created with destinations
            // that are unexplored, multiple expeditions may have the
            // same destination. Therefore, the destination needs to be
            // checked again here when the expedition is carried out.
            if visited.contains(&exp.destination) {
                continue;
            }

            // Update inlets + outlets of origin and destination nodes.
            if exp.origin != usize::MAX {
                // Update destination
                nodes[exp.destination].outlet = exp.origin;

                // Update origin.
                let origin = &mut nodes[exp.origin];
                debug_assert_eq!(origin.inlets[1], usize::MAX);
                let i = (origin.inlets[0] != usize::MAX) as usize;
                origin.inlets[i] = exp.destination;
            }

            let destination = &nodes[exp.destination];
            expand_bounds(destination.uv, &mut low_corner, &mut high_corner);
            visited.insert(exp.destination);

            // Create new expeditions originating from the newly
            // explored node to any unexplored neighbors.
            create_expeditions(
                &destination,
                nodes,
                &visited,
                &mut expeditions,
            );
        }

        GenerationInfo {
            mouths,
            low_corner,
            high_corner,
        }
    }

    /// Finds river mouth nodes.
    ///
    /// River mouth nodes are nodes that are within an ocean but which
    /// are adjacent to one or more nodes on land.
    ///
    /// # Arguments
    /// * `nodes` River nodes to search for mouths.
    ///
    /// # Return
    /// Vec of river node indices that are river mouths.
    fn find_mouths(nodes: &Vec<Node>) -> Vec<usize> {
        let mut mouths = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            // If node is not in an ocean, continue search.
            if node.h >= 0.0 {
                continue;
            }

            // Check if any neighbor is on land.
            for neighbor in &node.neighbors {
                if *neighbor != usize::MAX && nodes[*neighbor].h >= 0.0 {
                    mouths.push(i);
                    break;
                }
            }
        }

        mouths
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
