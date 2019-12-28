//! Todo:
//!
//! ! Add Navigable trait.
//! Implement Navigable types:
//!     StreetSegment
//!     CityRiverSegment
//!     BuildingSegment
//! Develop trait-based parameters & callback.
//! Implement TownMap.add(o: Builder)
//! Ensure serializable.
//! Implement village test.
//! Implement graph visualization.
//!
//! Initial goal is only to produce a street map.
//! Additional features will be implemented only after that.
use std::f64;
use std::usize;

use quad::{QuadMap, Rect, Spatial, ItemId};
use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;
use cgmath::MetricSpace;
use serde::{Deserialize, Serialize};

use pop::streets::builder::Builder;
use pop::streets::tensor::TensorField;


#[derive(
    Eq, PartialEq, Ord, PartialOrd,
    Hash, Clone, Copy, Debug,
    Serialize, Deserialize
)]
pub struct ObstacleId(ItemId);

#[derive(
    Eq, PartialEq, Ord, PartialOrd,
    Hash, Clone, Copy, Debug,
    Serialize, Deserialize
)]
pub struct NodeId(ItemId);

#[derive(
    Eq, PartialEq, Ord, PartialOrd,
    Hash, Clone, Copy, Debug,
    Serialize, Deserialize
)]
pub struct EdgeId(ItemId);


/// Map containing town map information.
///
/// Map is composed of three basic components:
///  * Obstacles.
///  * Nodes.
///  * Edges.
///
/// More complex components are managed by the TownPlan struct.
#[derive(Serialize)]
pub struct TownMap {
    nodes: QuadMap<Node>,
    edges: QuadMap<Edge>,
    obstacles: QuadMap<ObstacleLine>,
    value_map: TensorField,
    // 3do?

    settings: TownMapSettings,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    uv: Vector2<f64>,
    edges: Vec<(EdgeId, NodeId, Vector2<f64>)>,
    i: Option<NodeId>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    cost: f64,  // Travel cost of edge. Lower is better.
    a: NodeId,
    b: NodeId,
    uv_a: Vector2<f64>,
    uv_b: Vector2<f64>,
    bounds: Rect,
    i: Option<EdgeId>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObstacleLine {
    a: Vector2<f64>,
    b: Vector2<f64>,
    bounds: Rect,
    i: Option<ObstacleId>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TownMapSettings {
    pub node_merge_dist: f64,
    // As const settings are required, they should be added here.
}


// Implementation


impl TownMap {

    const DEFAULT_SHAPE: Rect = Rect::from_min_max(
        vec2(-3000.0, -3000.0),
        vec2(3000.0, 3000.0)
    );

    const DEFAULT_SETTINGS: TownMapSettings = TownMapSettings {
        node_merge_dist: 0.1,
    };

    // Construction

    /// Produce new StreetMap.
    ///
    /// # Arguments
    /// * `settings` - TownMapSettings with immutable settings which
    ///             will be kept for the lifetime of the StreetMap.
    ///
    /// # Return
    /// StreetMap
    pub fn new(settings: TownMapSettings) -> TownMap {
        TownMap {
            nodes: QuadMap::default(Self::DEFAULT_SHAPE),
            edges: QuadMap::default(Self::DEFAULT_SHAPE),
            obstacles: QuadMap::default(Self::DEFAULT_SHAPE),
            value_map: TensorField::new(Self::DEFAULT_SHAPE),
            settings,
        }
    }

    pub fn default() -> TownMap {
        Self::new(Self::DEFAULT_SETTINGS)
    }

    // Addition methods.

    /// Applies a builder to the map.
    ///
    /// This allows a builder instance to modify the map, usually to
    /// add a form of construct to the map - with nodes, edges,
    /// obstacle lines, and other features.
    ///
    /// # Arguments
    /// * `obj` - Object to add to the map.
    pub fn add<I>(&mut self, obj: &mut I)
        where I: Builder {
        // This function exists only for convenience, and invokes the
        // Builder. It may be removed.
        obj.build(self);
    }

    /// Adds a node to the StreetMap.
    ///
    /// If the passed node is near a pre-existing node, it will be
    /// merged with the existing node, and the Id of the pre-existing
    /// node will be returned.
    ///
    /// If no pre-existing node is near the added node, then the node
    /// will be added to the StreetMap and its id returned.
    ///
    /// # Arguments
    /// * `node` - Node reference to be added.
    ///
    /// # Return
    /// NodeId pointing to added node, or existing nearby node which
    /// should be used instead.
    pub fn add_node(&mut self, node: Node) -> &Node {
        {
            let existing = self.find_nearest_node(
                node.uv, self.settings.node_merge_dist
            );
            if existing.is_some() {
                return &self.nodes[existing.unwrap().0.id().0];
            }
        }

        let i = self.nodes.insert(node);
        self.nodes[i].i = Some(NodeId(i));

        &self.nodes[i]
    }

    /// Adds an edge to the street map.
    ///
    /// Both a and b node id's are expected to be valid.
    ///
    /// # Arguments:
    /// * `a` - NodeId of Node at one side of Edge.
    /// * `b` - NodeId of Node at other side of Edge.
    ///
    /// # Return
    /// EdgeId of added Edge.
    pub fn add_edge_between(&mut self, a: NodeId, b: NodeId, cost: f64) -> &Edge {
        let mut edge;
        {
            let a_node = self.node(a);
            let b_node = self.node(b);

            // Add connection to nodes.
            debug_assert!(!a_node.has_node_connection(b_node.id()));
            debug_assert!(!b_node.has_node_connection(a_node.id()));

            edge = Edge::new(a_node, b_node, cost);
        }

        let i = EdgeId(self.edges.insert(edge));
        self.edges[i.0].i = Some(i);
        let edge = &self.edges[i.0];
        self.nodes[a.0].add_edge(edge);
        self.nodes[b.0].add_edge(edge);

        edge
    }

    /// Adds obstacle line to the street map.
    ///
    /// # Arguments
    /// * `obstacle` - ObstacleLine to add to TownMap
    ///
    /// # Return
    /// Reference to ObstacleLine instance added to the map.
    pub fn add_obstacle(&mut self, obstacle: ObstacleLine) -> &ObstacleLine {
        let i = self.obstacles.insert(obstacle);
        self.obstacles[i].i = Some(ObstacleId(i));
        &self.obstacles[i]
    }

    // Accessors

    /// Get Node from NodeId
    ///
    /// # Arguments
    /// * `id` - NodeId specifying a Node
    ///
    /// # Return
    /// Node
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub fn nodes(&self) -> &QuadMap<Node> {
        &self.nodes
    }

    /// Gets node nearest to a set of UV coordinates within a radius.
    ///
    /// # Arguments
    /// * `uv` - Vector2<f64> specifying the center of the search area.
    /// * `r` - Radius around the position specified by `uv` within
    ///             which to search for the nearest Node.
    ///
    /// # Returns
    /// Tuple of:
    /// * Reference to the nearest node.
    /// * Distance to the nearest node.
    /// * NodeId of the nearest node.
    pub fn find_nearest_node(
        &self, uv: Vector2<f64>, r: f64
    ) -> Option<(&Node, f64)> {
        let (node, rect, id, d) = self.nodes.nearest(uv, r)?;
        Some((node, d))
    }

    pub fn obstacle_at(&self, id: ObstacleId) -> Option<&ObstacleLine> {
        self.obstacles.get(id.0)
    }

    pub fn edge_at(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(id.0)
    }

    pub fn edge(&self, id: EdgeId) -> &Edge {
        &self.edges[id.0]
    }

    pub fn value_map(&self) -> &TensorField {
        &self.value_map
    }
}

impl Node {
    pub fn new(uv: Vector2<f64>) -> Node {
        Node {
            uv,
            edges: Vec::with_capacity(4),
            i: None,
        }
    }

    pub fn has_node_connection(&self, id: NodeId) -> bool {
        for (_, node_id, _) in &self.edges {
            if *node_id == id {
                return true;
            }
        }
        false
    }

    pub fn has_edge(&self, id: EdgeId) -> bool {
        for (edge_id, node_id, uv) in &self.edges {
            if *edge_id == id {
                return true;
            }
        }
        false
    }

    pub fn add_edge(&mut self, edge: &Edge) {
        debug_assert!(self.has_id());
        debug_assert!(edge.has_id());
        debug_assert!(edge.a == self.id() || edge.b == self.id());
        debug_assert_ne!(edge.a, edge.b);

        let other_id = if edge.a == self.id() { edge.b } else { edge.a };
        let other_uv = if edge.a == self.id() { edge.uv_b } else { edge.uv_a };

        self.edges.push((edge.id(), other_id, other_uv));
    }

    // Info

    /// Checks whether node has > 3  edges.
    pub fn is_intersection(&self) -> bool {
        self.edges.len() >= 3
    }

    /// Checks whether node has two edges.
    pub fn is_pass_through(&self) -> bool {
        self.edges.len() == 2
    }

    /// Checks if node has only a single edge leading to it.
    pub fn is_end(&self) -> bool {
        self.edges.len() == 1
    }

    /// Checks if node has no edges connected to it.
    pub fn is_unconnected(&self) -> bool {
        self.edges.len() == 0
    }

    /// Checks if node is a pass-through node with angle < 45 deg.
    pub fn is_straight(&self) -> bool {
        if !self.is_pass_through() {
            return false;
        }
        let a_vec = (self.uv - self.edges[0].2).normalize();
        let b_vec = (self.edges[1].2 - self.uv).normalize();
        let cos = a_vec.dot(b_vec);
        cos > f64::consts::FRAC_1_SQRT_2
    }

    /// Checks if node is a passF-through node with angle > 45 deg.
    pub fn is_corner(&self) -> bool {
        self.is_pass_through() && !self.is_straight()
    }

    // Accessors

    pub fn id(&self) -> NodeId {
        self.i.unwrap()
    }

    pub fn has_id(&self) -> bool {
        self.i.is_some()
    }

    pub fn uv(&self) -> Vector2<f64> {
        self.uv
    }
}

impl Spatial for Node {
    fn aabb(&self) -> Rect {
        Rect::null_at(self.uv)
    }
}


impl ObstacleLine {
    pub fn new(a: Vector2<f64>, b: Vector2<f64>) -> ObstacleLine {
        ObstacleLine {
            a,
            b,
            bounds: Rect::from_points(a, b),
            i: None,
        }
    }
}


impl Spatial for ObstacleLine {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}


impl Edge {
    pub fn new(a: &Node, b: &Node, cost: f64) -> Edge {
        debug_assert!(a.has_id());
        debug_assert!(b.has_id());

        Edge {
            cost,
            a: a.id(),
            b: b.id(),
            uv_a: a.uv,
            uv_b: b.uv,
            bounds: Rect::from_points(a.uv, b.uv),
            i: None,
        }
    }

    // Info

    /// Produces angle in radians between one Edge and another.
    ///
    /// The returned value will always be between 0.0 and pi/2, since
    /// edges are all considered bidirectional.
    ///
    /// # Arguments
    /// * `other` - Edge to compare with.
    ///
    /// # Return
    /// Angle between first Edge and another, between 0.0 and pi/2.0.
    pub fn angle(&self, other: &Self) -> f64 {
        self.cos(other).acos()
    }

    /// Produces cosign of angle between one Edge and another.
    ///
    /// The returned value will always be between 0.0 and 1.0, since
    /// edges are all considered bidirectional.
    ///
    /// # Arguments
    /// * `other` - Edge to compare with.
    ///
    /// # Return
    /// Cosign of angle between first Edge and another, between 0.0
    /// and 1.0.
    pub fn cos(&self, other: &Self) -> f64 {
        self.dir().dot(other.dir()).abs()
    }

    // Accessors

    pub fn id(&self) -> EdgeId {
        self.i.unwrap()
    }

    pub fn has_id(&self) -> bool {
        self.i.is_some()
    }

    /// Gets direction vector of an Edge.
    ///
    /// Although the direction vector of a given Edge will be
    /// consistent, edges should be considered to be bidirectional.
    pub fn dir(&self) -> Vector2<f64> {
        (self.uv_b - self.uv_a).normalize()
    }
}


impl Spatial for Edge {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}


#[cfg(test)]
mod tests {
    use std::f64;

    use cgmath::vec2;
    use assert_approx_eq::assert_approx_eq;

    use pop::streets::builder::StreetSegmentBuilder;
    use pop::streets::map::{TownMap, TownMapSettings, Node, Edge};

    // ----------------------------
    // TownMap

    /// Test that the nearest node to a passed position can be found.
    #[test]
    fn test_find_nearest_node() {
        let mut map = TownMap::default();

        map.add_node(Node::new(vec2(0.0, 1000.0)));
        map.add_node(Node::new(vec2(0.0, 0.0)));  // Should be nearest.
        map.add_node(Node::new(vec2(1000.0, 0.0)));
        map.add_node(Node::new(vec2(-500.0, -500.0)));
        map.add_node(Node::new(vec2(100.0, -200.0)));
        map.add_node(Node::new(vec2(-200.0, 100.0)));

        let (node, d) =
            map.find_nearest_node(vec2(200.0, 200.0), 300.0).unwrap();

        assert_vec2_near!(node.uv, vec2(0.0, 0.0));
    }

    /// Test that the nearest node to a passed position is not returned
    /// if the radius is too small.
    #[test]
    fn test_find_nearest_node_returns_none_if_radius_too_small() {
        let mut map = TownMap::default();

        map.add_node(Node::new(vec2(0.0, 1000.0)));
        map.add_node(Node::new(vec2(0.0, 0.0)));  // Nearest.
        map.add_node(Node::new(vec2(1000.0, 0.0)));

        assert!(map.find_nearest_node(vec2(200.0, 200.0), 220.0).is_none());
    }

    /// Test node is not added if an existing node is at the
    /// same location.
    #[test]
    fn test_add_node() {
        let mut map = TownMap::default();

        let a = map.add_node(Node::new(vec2(0.0, 1000.0))).id();
        let b = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(0.01, 0.05))).id();

        assert_ne!(a, b);
        assert_eq!(b, c);
    }

    // ----------------------------
    // Node

    #[test]
    fn test_node_is_straight() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(3.0, 1.0))).id();
        let ab = map.add_edge_between(a, b, 1.0);
        let bc = map.add_edge_between(b, c, 1.0);
        assert!(map.node(b).is_straight());
    }

    #[test]
    fn test_node_is_straight_is_false_when_reversal() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(-3.0, 1.0))).id();
        let ba = map.add_edge_between(a, b, 1.0);
        let bc = map.add_edge_between(b, c, 1.0);
        assert!(!map.node(b).is_straight());
    }

    #[test]
    fn test_node_is_straight_is_false_when_right_angle() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(1.0, -1.1))).id();
        let ab = map.add_edge_between(a, b, 1.0);
        let bc = map.add_edge_between(b, c, 1.0);
        assert!(!map.node(b).is_straight());
    }

    #[test]
    fn test_node_is_corner_when_right_angle() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(1.0, -1.1))).id();
        let ab = map.add_edge_between(a, b, 1.0);
        let bc = map.add_edge_between(b, c, 1.0);
        assert!(map.node(b).is_corner());
    }

    #[test]
    fn test_node_is_corner_false_when_intersection() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(1.0, -1.1))).id();
        let d = map.add_node(Node::new(vec2(1.0, 1.0))).id();
        let ab = map.add_edge_between(a, b, 1.0);
        let bc = map.add_edge_between(b, c, 1.0);
        let bd = map.add_edge_between(b, d, 1.0);
        assert!(!map.node(b).is_corner());
    }

    #[test]
    fn test_node_is_straight_false_when_end() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let ab = map.add_edge_between(a, b, 1.0);
        assert!(!map.node(b).is_straight());
    }

    // ----------------------------
    // Edge

    #[test]
    fn test_edge_angle_straight() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(3.0, 0.0))).id();
        let ab = map.add_edge_between(a, b, 1.0).id();
        let ac = map.add_edge_between(a, c, 1.0).id();

        let ab = map.edge(ab);
        let ac = map.edge(ac);
        assert_approx_eq!(ab.angle(&ac), 0.0)
    }

    #[test]
    fn test_edge_angle_45() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(1.0, 1.0))).id();
        let ab = map.add_edge_between(a, b, 1.0).id();
        let ac = map.add_edge_between(a, c, 1.0).id();

        let ab = map.edge(ab);
        let ac = map.edge(ac);
        assert_approx_eq!(ab.angle(&ac),  f64::consts::PI / 4.0);
    }

    #[test]
    fn test_edge_angle_right() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(0.0, 1.0))).id();
        let ab = map.add_edge_between(a, b, 1.0).id();
        let ac = map.add_edge_between(a, c, 1.0).id();

        let ab = map.edge(ab);
        let ac = map.edge(ac);
        assert_approx_eq!(ab.angle(ac),  f64::consts::PI / 2.0);
    }

    #[test]
    fn test_edge_angle_135() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(-1.0, 1.0))).id();
        let ab = map.add_edge_between(a, b, 1.0).id();
        let ac = map.add_edge_between(a, c, 1.0).id();

        let ab = map.edge(ab);
        let ac = map.edge(ac);
        assert_approx_eq!(ab.angle(&ac),  f64::consts::PI / 4.0);
    }
}
