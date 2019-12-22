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

use std::usize;

use quad::{QuadMap, Rect, Spatial, ItemId};
use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;
use cgmath::MetricSpace;

use pop::streets::builder::Builder;
use pop::streets::tensor::TensorField;


#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct ObstacleId(ItemId);

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct NodeId(ItemId);

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct EdgeId(ItemId);


/// Map containing town map information.
///
/// Map is composed of three basic components:
///  * Obstacles.
///  * Nodes.
///  * Settings.
pub struct TownMap {
    nodes: QuadMap<Node>,
    edges: QuadMap<Edge>,
    obstacles: QuadMap<ObstacleLine>,
    value_map: TensorField,

    settings: TownMapSettings,
}


#[derive(Clone, Debug)]
pub struct Node {
    uv: Vector2<f64>,
    edges: Vec<(NodeId, EdgeId)>,
    i: Option<NodeId>,
}


#[derive(Clone, Debug)]
pub struct Edge {
    cost: f64,  // Travel cost of edge. Lower is better.
    a: NodeId,
    b: NodeId,
    uv_a: Vector2<f64>,
    uv_b: Vector2<f64>,
    bounds: Rect,
    i: Option<EdgeId>,
}


#[derive(Clone, Debug)]
pub struct ObstacleLine {
    a: Vector2<f64>,
    b: Vector2<f64>,
    bounds: Rect,
    i: Option<ObstacleId>,
}


#[derive(Clone, Debug)]
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

    /// Adds passed street map object to the map.
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
    pub fn add_node(&mut self, mut node: Node) -> NodeId {
        {
            let existing = self.find_nearest_node(
                node.uv, self.settings.node_merge_dist
            );
            if existing.is_some() {
                return existing.unwrap().0.id();
            }
        }

        let i = self.nodes.insert(node);
        self.nodes[i].i = Some(NodeId(i));

        NodeId(i)
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

        self.nodes[a.0].add_edge(&edge);
        self.nodes[b.0].add_edge(&edge);

        let i = self.edges.insert(edge);
        self.edges[i].i = Some(EdgeId(i));

        &self.edges[i]
    }

    /// Adds obstacle line to the street map.
    pub fn add_obstacle(&mut self, mut obstacle: ObstacleLine) -> &ObstacleLine {
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
        for (node_id, edge_id) in &self.edges {
            if *node_id == id {
                return true;
            }
        }
        false
    }

    pub fn has_edge(&self, id: EdgeId) -> bool {
        for (node_id, edge_id) in &self.edges {
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

        let other_id = if edge.a == self.id() { edge.b } else { edge.a };

        self.edges.push((other_id, edge.id()));
    }

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
            b: a.id(),
            uv_a: a.uv,
            uv_b: b.uv,
            bounds: Rect::from_points(a.uv, b.uv),
            i: None,
        }
    }

    pub fn id(&self) -> EdgeId {
        self.i.unwrap()
    }

    pub fn has_id(&self) -> bool {
        self.i.is_some()
    }
}


impl Spatial for Edge {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}


#[cfg(test)]
mod tests {
    use cgmath::vec2;

    use pop::streets::builder::StreetSegmentBuilder;
    use pop::streets::map::{TownMap, TownMapSettings, Node};

    // ----------------------------
    // StreetMap

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

        let a = map.add_node(Node::new(vec2(0.0, 1000.0)));
        let b = map.add_node(Node::new(vec2(0.0, 0.0)));
        let c = map.add_node(Node::new(vec2(0.01, 0.05)));

        assert_ne!(a, b);
        assert_eq!(b, c);
    }
}
