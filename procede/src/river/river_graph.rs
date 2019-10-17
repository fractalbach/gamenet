use std::cmp::Ordering;
use std::collections::{VecDeque, HashSet, BinaryHeap};
use std::f64;
use std::num::Wrapping;
use std::usize;

use aabb_quadtree::QuadTree;
use aabb_quadtree::geom::Rect;
use cgmath::Vector2;
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::ser::SerializeStruct;

use util::{idx_hash, rand1};
use serde_util::SerializableVector2;
use river::common::{get_base_width, vec2pt};
use river::segment::Segment;

pub struct RiverGraph {
    pub segment_tree: QuadTree<Segment>,
    pub nodes: Vec<Node>
}

/// River node
pub struct Node {
    pub i: usize,  // Index of Node in river graph.
    pub indices: Vector2<i64>,
    pub uv: Vector2<f64>,
    pub h: f64,  // Height above mean sea level.
    pub neighbors: [usize; 3],  // Neighboring nodes within graph. Clockwise.
    pub inlets: [usize; 2],
    pub outlet: usize,
    pub direction: Vector2<f64>,
    pub fork_angle: f64,
    pub strahler: i8
}

pub struct GenerationInfo {
    pub low_corner: Vector2<f64>,
    pub high_corner: Vector2<f64>,
}


// --------------------------------------------------------------------


impl RiverGraph {
    pub fn new(mut nodes: Vec<Node>, mouths: &Vec<usize>) -> RiverGraph {
        // Connect nodes in-place.
        let info = Self::generate_rivers(&mut nodes, mouths);
        Self::find_direction_info(&mut nodes);

        // Create bounding shape
        let shape = Rect{
            top_left: vec2pt(info.low_corner),
            bottom_right: vec2pt(info.high_corner),
        };

        // Create river segments
        let segment_tree = Self::create_segments(&nodes, mouths, shape);

        RiverGraph {
            nodes,
            segment_tree
        }
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
    fn generate_rivers(nodes: &mut Vec<Node>, mouths: &Vec<usize>) -> GenerationInfo {
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

        let mut expeditions = BinaryHeap::with_capacity(100);
        let mut visited = HashSet::with_capacity(100);

        let mut low_corner = Vector2::new(0.0, 0.0);
        let mut high_corner = Vector2::new(0.0, 0.0);

        // Initialize expeditions with river mouths
        for mouth in mouths {
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
            low_corner,
            high_corner,
        }
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
}

impl Serialize for RiverGraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_some(&self.nodes)
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
    pub fn new(
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
    ///
    /// # Arguments
    /// * `node_i` - Index of inlet Node in river Node vec.
    ///
    /// # Return
    /// Index in inlets array. 0 or 1. (0 is left, 1 is right).
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
    pub fn is_fork(&self) -> bool {
        self.inlets[1] != usize::MAX
    }

    /// Gets left side of fork
    ///
    /// Left and right inlets are based on the perspective of an
    /// observer facing upstream.
    pub fn left_inlet(&self) -> usize {
        debug_assert!(self.is_fork());
        self.inlets[0]
    }

    /// Gets right side of fork
    ///
    /// Left and right inlets are based on the perspective of an
    /// observer facing upstream.
    pub fn right_inlet(&self) -> usize {
        debug_assert!(self.is_fork());
        self.inlets[1]
    }

    /// Checks if node is a river mouth (has no outlet)
    pub fn is_mouth(&self) -> bool {
        self.outlet == usize::MAX
    }

    /// Gets width of river at node.
    ///
    /// If river is a fork, this value may not reflect the observed
    /// width at the node.
    pub fn width(&self) -> f64 {
        // May be replaced later if node widths are blended.
        get_base_width(self.strahler)
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        let mut state = serializer.serialize_struct("RiverNode", 10)?;
        state.serialize_field("i", &self.i)?;
        let ser_indices = SerializableVector2::new(&self.indices);
        state.serialize_field("indices", &ser_indices)?;
        let ser_uv = SerializableVector2::new(&self.uv);
        state.serialize_field("uv", &ser_uv)?;
        state.serialize_field("h", &self.h)?;
        state.serialize_field("neighbors", &self.neighbors)?;
        state.serialize_field("inlets", &self.inlets)?;
        state.serialize_field("outlet", &self.outlet)?;
        let ser_direction = SerializableVector2::new(&self.direction);
        state.serialize_field("direction", &ser_direction)?;
        state.serialize_field("fork_angle", &self.fork_angle)?;
        state.serialize_field("strahler", &self.strahler)?;
        state.end()
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use cgmath::Vector2;

    use river::river_graph::*;

    // ----------------------------------------------------------------
    // RiverGraph


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
}
