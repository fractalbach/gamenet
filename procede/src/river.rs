//! Module containing river procedural generation structs
//! and functions.
use std::f64;
use std::num::Wrapping;

use aabb_quadtree::{QuadTree, ItemId, Spatial};
use aabb_quadtree::geom::{Rect, Point};
use cgmath::{Vector2, Vector3};
use cgmath::InnerSpace;
use lru_cache::LruCache;
use std::cmp::Ordering;
use std::collections::{VecDeque, HashSet, HashMap, BinaryHeap};
use std::usize;

use tectonic::{TectonicLayer, TectonicInfo};
use util::{rand1, hash_indices, sphere_uv_vec, idx_hash};


// --------------------------------------------------------------------


/// Struct handling generation of major river effects on the map
///
/// The RiverInfo produced by this layer includes an updated height
/// value that includes the effects of river valley formation, as well
/// as other major-river related information.
///
/// The .height() method is the main public method exposed by
/// a RiverLayer. When passed a position, and TectonicInfo struct, the
/// RiverLayer will generate or retrieve a river Region, and then yield
/// a RiverInfo struct instance for that position.
pub struct RiverLayer {
    seed: u32,
    region_cache: LruCache<Vector3<i64>, Region>,
}

/// Struct used to return height and related information about
/// a position from the RiverLayer.
pub struct RiverInfo {
    height: f64,
}

/// River region.
///
/// A River region is associated with a single tectonic cell, and
/// handles height generation due to river action within its bounds.
///
/// A Tectonic cell is an ideal boundary for a river region because it
/// is likely to either border an ocean, or else be bordered by a
/// mountain range which would realistically separate river basins.
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
    neighbors: [usize; 3],  // Neighboring nodes within graph. Clockwise.
    inlets: [usize; 2],
    outlet: usize,
    direction: Vector2<f64>,
    fork_angle: f64,
    strahler: i8
}

struct GenerationInfo {
    mouths: Vec<usize>,
    low_corner: Vector2<f64>,
    high_corner: Vector2<f64>,
}

/// River segment joining two nodes.
///
/// Handles calculation of river info based on distance to the
/// Segment's river course.
///
/// The Segment will handle any blending of data from different curves,
struct Segment {
    base_curve: Curve,
    bounds: Rect,
    upriver_w: f64,
    downriver_w: f64,
}

/// A single river bezier curve.
///
/// Handles calculation of a point's distance to a curve.
struct Curve {
    a: Vector2<f64>,
    ctrl_a: Vector2<f64>,
    ctrl_b: Vector2<f64>,
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

    fn new(seed: u32) -> RiverLayer {
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
        let region = self.region_cache.get_mut(&indices).unwrap();
        region.height(v)
    }
}


// --------------------------------------------------------------------


impl Region {
    pub const NODE_MEAN_SEPARATION: f64 = 10_000.0;
    const CONTROL_POINT_DIST: f64 = Self::NODE_MEAN_SEPARATION * 0.2;

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

        // Create river nodes.
        let mut nodes = Self::create_nodes(seed, tectonic, tectonic_info);

        // Connect nodes in-place.
        let info = Self::generate_rivers(&mut nodes);
        Self::find_direction_info(&mut nodes);

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
        /// Finds the first hex index contained within the cell.
        ///
        /// # Arguments
        /// * `tectonic` - Tectonic layer whose cell is being explored.
        /// * `cell_indices` - Indices of cell being explored.
        /// * `hex_graph`- HexGraph used to generate base
        ///             node positions.
        ///
        /// # Return
        /// HexGraph indices of node within the cell from which
        /// exploration will start.
        fn find_first(
            tectonic: &mut TectonicLayer,
            cell_indices: Vector3<i64>,
            hex_graph: &HexGraph
        ) -> Vector2<i64> {
            // TODO: Ensure first node is in cell. If not, do quick search.
            Vector2::new(0, 0)
        }

        /// Runs BFS Search until all nodes that are in cell are added
        /// to nodes Vec.
        ///
        /// Included indices are added to included set for
        /// quick checking.
        ///
        /// # Arguments
        /// * `tectonic` - Reference to tectonic layer used to generate
        ///             heights for nodes.
        /// * `cell_indices` - Indices of cell which is being explored.
        /// * `hex_graph` - HexGraph used to generate nodes and lookup
        ///             node positions.
        /// * `first` - HexGraph indices of node at which to
        ///             start exploration.
        ///
        /// # Return
        /// * Vec of Nodes which are contained by cell.
        /// * HashMap with the index of each node in the Node Vec,
        ///             stored with the node's HexGraph indices as key.
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
                nodes.push(Node::new(
                    node_i,
                    indices,
                    uv,
                    node_info.height,
               ));

                // Add hex neighbors to frontier.
                for hex_neighbor in &hex_graph.neighbors(indices) {
                    if !visited.contains(&hex_neighbor) {
                        frontier.push_back(*hex_neighbor);
                    }
                }
            }

            (nodes, included)
        }

        /// Sets node neighbors.
        ///
        /// These are the nodes which are within the cell. If a cell has
        /// fewer than three neighbors, one or more index will be set to
        /// -1 (since usize is unsigned, this will be usize::MAX)
        ///
        /// The nodes within the neighbors are ordered clockwise.
        ///
        /// This function modifies the nodes in-place. It does not
        /// return a useful value.
        ///
        /// # Arguments
        /// * `nodes` - Vec of nodes in cell.
        /// * `included` - Map of Node Vec indices stored by their
        ///             HexGraph indices.
        /// * `hex_graph` - HexGraph used to generate nodes.
        ///
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
    /// No node will have more than two inlets, even in the possible
    /// but unlikely case where the node is a river mouth that has
    /// three neighbors which are valid as inlets.
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
                // Ignore unused neighbor slots.
                if *neighbor == usize::MAX {
                    continue;
                }

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
                // Handle case where origin already has two inlets.
                // This is rare, but possible for river mouths.
                if nodes[exp.destination].inlets[1] != usize::MAX {
                    continue;
                }
                // Update destination
                nodes[exp.destination].outlet = exp.origin;

                // Update origin.
                nodes[exp.origin].add_inlet(exp.destination);
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

    /// Sets node information relating to river direction.
    ///
    /// Node direction (direction vector of the node's output) and fork
    /// angle (angle between input rivers at the  point of
    /// intersection) are set for each node in the nodes vector.
    ///
    /// This method does not return useful values. Instead, the passed
    /// nodes vector is modified in place.
    ///
    /// # Arguments
    /// * `nodes` - Vector of Nodes.
    fn find_direction_info(nodes: &mut Vec<Node>) {
        /// Finds outlet direction of node.
        ///
        /// This will determine the direction of segment control points
        /// from the node.
        ///
        /// This will differ from the direction of the outlet
        /// node from the passed node.
        ///
        /// If no outlet exists (the node is a river mouth), then the
        /// outlet direction will be (0.0, 0.0).
        ///
        /// # Arguments
        /// * `i` - Index of node whose outlet direction will
        ///             be determined.
        /// * `nodes` - Reference to Node Vec.
        ///
        /// # Return
        /// Vector2 with direction of outlet from node if node has an
        /// outlet, otherwise (0.0, 0.0)
        fn find_outlet_dir(i: usize, nodes: &mut Vec<Node>) -> Vector2<f64> {
            let node = &nodes[i];
            if node.outlet == usize::MAX {
                return Vector2::new(0.0, 0.0);
            }

            let node_pos = node.uv;
            let outlet_pos = nodes[node.outlet].uv;

            let direction = (outlet_pos - node_pos).normalize();

            direction
        }

        /// Finds the angle between node inlets.
        ///
        /// This determines the acuteness of the river fork located at
        /// the passed node.
        ///
        /// If a node has only a single inlet, then the returned value
        /// will be 0.0.
        ///
        /// # Arguments
        /// * `i` - Index of node whose outlet direction will
        ///             be determined.
        /// * `nodes` - Reference to Node Vec.
        ///
        /// # Return
        /// Angle of fork in radians.
        fn find_fork_angle(i: usize, nodes: &mut Vec<Node>) -> f64 {
            let node = &nodes[i];
            if node.inlets[1] == usize::MAX {
                return 0.0;
            }

            // Generate random value between 20 and 50.
            const MIN_ANGLE: f64 = f64::consts::PI / 9.0;
            const MAX_ANGLE: f64 = f64::consts::PI / 3.5;
            const RANGE: f64 = MAX_ANGLE - MIN_ANGLE;
            const MEAN_ANGLE: f64 = MIN_ANGLE + RANGE / 2.0;

            let rand = rand1(idx_hash(i as i64));
            let angle = RANGE / 2.0 * rand + MEAN_ANGLE;

            angle
        }

        // ------------------------

        for i in 0..nodes.len() {
            let direction = find_outlet_dir(i, nodes);
            let fork_angle = find_fork_angle(i, nodes);

            let node = nodes.get_mut(i).expect("Invalid node index");
            node.direction = direction;
            node.fork_angle = fork_angle;
        }
    }

    /// Create river segments from nodes.
    ///
    /// A Segment is composed of four points, which form a bezier curve.
    /// These nodes are the Up-river end node, the Up-river control
    /// node, the Down-river end node, and the Down-river control node.
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
        let mut tree: QuadTree<Segment> = QuadTree::default(shape);
        let mut frontier: VecDeque<usize> = VecDeque::with_capacity(100);

        // Add mouths to `frontier` to-do list.
        for i in mouths {
            frontier.push_back(*i);
        }

        // Progress up-river creating segments.
        while !frontier.is_empty() {
            let i = frontier.pop_front().unwrap();
            let node = &nodes[i];

            // Add inlets to frontier.
            for inlet in &node.inlets {
                if *inlet != usize::MAX {
                    frontier.push_back(*inlet);
                }
            }

            // If Node has an outlet, create segment connecting node to
            // outlet node. If node has no outlet, continue.
            if node.outlet == usize::MAX {
                continue;
            }

            // Create Segment.
            let downriver = &nodes[node.outlet];
            let segment = Segment::new(downriver, node);
            tree.insert(segment);
        }

        tree
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
            // TODO: Replace placeholder.
            Segment {
                base_curve: Curve {
                    a: Vector2::new(-1.0, -1.0),
                    ctrl_a: Vector2::new(-1.0, -1.0),
                    ctrl_b: Vector2::new(1.0, 1.0),
                    b: Vector2::new(1.0, 1.0)
                },
                bounds: Rect {
                    top_left: Point { x: 0.0, y: 0.0 },
                    bottom_right: Point { x: 0.0, y: 0.0 }
                },
                upriver_w: -1.0,
                downriver_w: -1.0,
            }
        )
    }
}


// --------------------------------------------------------------------


impl Node {
    /// Creates new Node.
    ///
    /// Takes as arguments values which are available at instantiation,
    /// and provides default values for the others.
    ///
    /// # Arguments
    /// * `i` - Index of node in nodes vector.
    /// * `indices` - HexGraph uv indices.
    /// * `uv` - Position on UV plane.
    /// * `h` - Height above mean sea level.
    ///
    /// # Return
    /// Partially initialized Node containing passed values.
    fn new(
            i: usize,
            indices: Vector2<i64>,
            uv: Vector2<f64>,
            h: f64
    ) -> Node {
        Node {
            i,  // Index of Node in river graph.
            indices,
            uv,
            h,  // Height above mean sea level.
            neighbors: [usize::MAX, usize::MAX, usize::MAX],
            inlets: [usize::MAX, usize::MAX],
            outlet: usize::MAX,
            direction: Vector2::new(0.0, 0.0),
            fork_angle: -1.0,
            strahler: -1
        }
    }

    /// Adds inlet node.
    fn add_inlet(&mut self, node_i: usize) -> usize {
        // Node should not already have two inlets.
        debug_assert!(!self.is_fork());

        // If no previous inlet exists, inlet index will be set in
        // inlets[0]. If a previous inlet exists, then the left
        // tributary index will be set in inlets[0], and the right
        // tributary index will be set in inlets[1].
        let inlet_index;
        if self.inlets[0] == usize::MAX {
            inlet_index = 0usize;
        } else {
            // Use neighbors array to find left/right ordering.
            // Since the neighbors array should be ordered clockwise,
            // it provides an efficient way to determine whether a node
            // is on the left or right.
            let mut inlet_i = usize::MAX;
            for i in 0..3 {
                if node_i == self.neighbors[i] {
                    inlet_i = i;
                    break;
                }
            }
            debug_assert!(inlet_i != usize::MAX);
            // Check if pre-existing inlet is on right side.
            if self.inlets[0] == self.neighbors[(inlet_i + 1) % 3] {
                inlet_index = 0;
                // Move old inlet to right side.
                self.inlets[1] = self.inlets[0]
            } else {
                inlet_index = 1;
            }
        }
        self.inlets[inlet_index] = node_i;
        inlet_index
    }

    // Getters

    /// Checks whether node is a fork.
    fn is_fork(&self) -> bool {
        self.inlets[1] != usize::MAX
    }

    /// Gets left side of fork
    ///
    /// Left and right inlets are based on the perspective of an
    /// observer facing upstream.
    fn left_inlet(&self) -> usize {
        debug_assert!(self.is_fork());
        self.inlets[0]
    }

    /// Gets right side of fork
    ///
    /// Left and right inlets are based on the perspective of an
    /// observer facing upstream.
    fn right_inlet(&self) -> usize {
        debug_assert!(self.is_fork());
        self.inlets[1]
    }

    /// Checks if node is a river mouth (has no outlet)
    fn is_mouth(&self) -> bool {
        self.outlet == usize::MAX
    }

    /// Gets width of river at node.
    ///
    /// If river is a fork, this value may not reflect the observed
    /// width at the node.
    fn width(&self) -> f64 {
        // May be replaced later if node widths are blended.
        get_base_width(self.strahler)
    }
}


// --------------------------------------------------------------------


impl Segment {
    const MAX_STRAHLER: i8 = 12;
    const MAX_MEANDER_BAND: f64 = get_base_width(Self::MAX_STRAHLER) * 20.0;
    const BASE_BOUND_MARGIN: f64 = Self::MAX_MEANDER_BAND * 2.0;
    const STRAHLER_INC_W_RATIO: f64 = 0.7;

    fn new(downriver: &Node, upriver: &Node) -> Segment {
        let base_curve = Curve {
            a: Vector2::new(-1.0, -1.0),
            ctrl_a: Vector2::new(-1.0, -1.0),
            ctrl_b: Vector2::new(1.0, 1.0),
            b: Vector2::new(1.0, 1.0)
        };
        let bounds = Self::find_bounds(&base_curve, Self::BASE_BOUND_MARGIN);
        let upriver_w = upriver.width();
        let downriver_w = if downriver.strahler > upriver.strahler {
            downriver.width()
        } else {
            downriver.width() * Self::STRAHLER_INC_W_RATIO
        };

        Segment {
            base_curve,
            bounds,
            upriver_w,
            downriver_w,
        }
    }

    // Constructor helpers.

    /// Finds downriver control node position.
    ///
    /// The down-river control node is the control node closer to
    /// the downriver of the endpoints of a segment.
    ///
    /// # Arguments
    /// * `node` - Reference to downriver node.
    /// * `i` - Index of the inlet node which this segment
    ///             connects to. Should be 0 or 1.
    ///
    /// # Return
    /// UV Position of the downriver control node.
    fn downriver_control_node(node: &Node, i: usize) -> Vector2<f64> {
        Vector2::new(0.0, 0.0)  // TODO
    }

    /// Finds up-river control node position.
    ///
    /// The up-river control node is the control node closer to
    /// the upriver node of a segment.
    ///
    /// # Arguments
    /// * `node` - Reference to up-river node.
    ///
    /// # Return
    /// UV Position of the up-river control node.
    fn upriver_control_node(node: &Node) -> Vector2<f64> {
        Vector2::new(0.0, 0.0)  // TODO
    }

    fn find_bounds(base_curve: &Curve, margin: f64) -> Rect {
        let mut min_x: f64 = base_curve.a.x;
        let mut max_x: f64 = base_curve.a.x;
        let mut min_y: f64 = base_curve.a.y;
        let mut max_y: f64 = base_curve.a.y;

        for point in &[base_curve.b, base_curve.ctrl_b, base_curve.ctrl_a] {
            if point.x < min_x {
                min_x = point.x;
            } else if point.x > max_x {
                max_x = point.x;
            }
            if point.y < min_y {
                min_y = point.y;
            } else if point.y > max_y {
                max_y = point.y;
            }
        }

        // Create Rect. Note that the top-left field contains the
        // minimums, due to the quad-tree library being intended for
        // 2d graphics applications.
        Rect {
            top_left: Point {
                x: (min_x - margin) as f32,
                y: (min_y + margin) as f32
            },
            bottom_right: Point {
                x: (max_x - margin) as f32,
                y: (max_y + margin) as f32
            }
        }
    }

    // Instance methods.

    /// Finds river info determined by the segment at a given position.
    ///
    /// # Arguments
    /// * `uv` - Position in UV-space.
    ///
    /// # Return
    /// RiverInfo determined by Segment.
    fn info(&self, uv: Vector2<f64>) -> RiverInfo {
        return RiverInfo { height: -1.0 }
    }

    /// Finds base river width at passed ratio of length.
    ///
    /// # Arguments
    /// * `ratio` - Ratio of distance upriver.
    ///             0.0 == downriver end, 1.0 == upriver end.
    ///
    /// # Return
    /// Base width of river at identified point.
    fn base_width(&self, ratio: f64) -> f64 {
        self.downriver_w * ratio + self.upriver_w * (1.0 - ratio)
    }
}

impl Spatial for Segment {
    fn aabb(&self) -> Rect {
        self.bounds
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
        // This statement is a workaround for the '%' operator,
        // which produces the remainder, rather than the modulo.
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
    /// Returned neighbors are clockwise-ordered.
    fn neighbors(&self, indices: Vector2<i64>) -> [Vector2<i64>; 3] {
        // Get index within 4 vertex sequence.
        // This statement is a workaround for the '%' operator,
        // which produces the remainder, rather than the modulo.
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
fn vec2pt(v: Vector2<f64>) -> Point {
    Point {
        x: v.x as f32,
        y: v.y as f32,
    }
}

const fn get_base_width(strahler: i8) -> f64 {
    // Width table based on real-world measurements.
    const LOOKUP: [f64; 13] = [
        1.0,  // 0
        1.5,  // 1
        2.0,  // 2
        5.0,  // 3
        10.0,  // 4
        50.0,  // 5
        100.0,  // 6
        180.0,  // 7
        400.0,  // 8
        800.0,  // 9
        1000.0,  // 10
        2000.0,  // 11
        4000.0,  // 12
    ];

    LOOKUP[strahler as usize]
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use cgmath::Vector2;
    use cgmath::MetricSpace;

    use river::*;

    // ----------------------------------------------------------------
    // Region

    #[test]
    fn test_find_mouths() {
        let nodes = vec!(
            Node {
                neighbors: [1, 2, 3],
                ..Node::new(
                    0,
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    -13.0
                )
            },
            Node {
                neighbors: [0, 2, usize::MAX],
                ..Node::new(
                    1,
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    -24.0
                )
            },
            Node {
                neighbors: [0, 1, 3],
                ..Node::new(
                    2,  // Index of Node in river graph.
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    -11.0,  // Height above mean sea level.
                )
            },
            Node {
                neighbors: [0, 2, usize::MAX],
                ..Node::new(
                    3,  // Index of Node in river graph.
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    18.0,  // Height above mean sea level.
                )
            }
        );

        let mouths = Region::find_mouths(&nodes);

        assert_eq!(mouths, vec!(0, 2));
    }

    // ----------------------------------------------------------------
    // Node

    #[test]
    fn test_node_add_inlet_handles_clockwise_addition() {
        let mut node = Node::new(
            0,
            Vector2::new(0, 1),
            Vector2::new(10.0, 0.0),
            124.0
        );

        node.neighbors = [10, 20, 30];
        node.add_inlet(20);
        node.add_inlet(30);

        assert_eq!(node.left_inlet(), 20);
        assert_eq!(node.right_inlet(), 30);
    }

    #[test]
    fn test_node_add_inlet_handles_clockwise_addition2() {
        let mut node = Node::new(
            0,
            Vector2::new(0, 1),
            Vector2::new(10.0, 0.0),
            124.0
        );

        node.neighbors = [10, 20, 30];
        node.add_inlet(30);
        node.add_inlet(10);

        assert_eq!(node.left_inlet(), 30);
        assert_eq!(node.right_inlet(), 10);
    }

    #[test]
    fn test_node_add_inlet_handles_counter_clockwise_addition() {
        let mut node = Node::new(
            0,
            Vector2::new(0, 1),
            Vector2::new(10.0, 0.0),
            124.0
        );

        node.neighbors = [10, 20, 30];
        node.add_inlet(20);
        node.add_inlet(10);

        assert_eq!(node.left_inlet(), 10);
        assert_eq!(node.right_inlet(), 20);
    }

    #[test]
    fn test_node_is_fork() {
        let mut node = Node::new(
            0,
            Vector2::new(0, 1),
            Vector2::new(10.0, 0.0),
            124.0
        );
        node.neighbors = [10, 20, 30];

        assert!(!node.is_fork());
        node.add_inlet(20);
        assert!(!node.is_fork());
        node.add_inlet(10);
        assert!(node.is_fork());
    }

    #[test]
    fn test_node_is_mouth() {
        let mut node = Node::new(
            0,
            Vector2::new(0, 1),
            Vector2::new(10.0, 0.0),
            124.0
        );
        node.neighbors = [10, 20, 30];

        assert!(node.is_mouth());
        node.outlet = 20;
        assert!(!node.is_mouth());
    }

    // ----------------------------------------------------------------
    // HexGraph

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

    // Test module level functions

    #[test]
    fn test_base_width() {
        assert_in_range!(0.75, get_base_width(0), 1.5);
        assert_in_range!(1.0, get_base_width(1), 2.0);
        assert_in_range!(5.0, get_base_width(4), 50.0);
        assert_in_range!(700.0, get_base_width(10), 2000.0);
        assert_in_range!(3000.0, get_base_width(12), 8000.0);
    }
}
