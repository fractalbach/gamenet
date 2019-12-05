
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

use aabb_quadtree::{QuadTree, ItemId};
use aabb_quadtree::geom::{Rect, Point};
use aabb_quadtree::Spatial;
use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;
use cgmath::MetricSpace;


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
pub struct StreetMap {
    obstacles: QuadTree<ObstacleLine>,
    nodes: QuadTree<Node>,
    edges: QuadTree<Edge>,
    settings: StreetSettings,
}


#[derive(Clone, Debug)]
struct Node {
    uv: Vector2<f64>,
    edges: Vec<(NodeId, EdgeId)>
}


#[derive(Clone, Debug)]
struct EdgeInfo {
    cost_mod: f64,  // Travel cost of edge. Lower is better.
    a: NodeId,
    b: NodeId,
}


#[derive(Clone, Debug)]
struct Edge {
    cost: f64,  // Travel cost of edge. Lower is better.
    a: NodeId,
    b: NodeId,
    a_uv: Vector2<f64>,
    b_uv: Vector2<f64>,
    bounds: Rect,
}


#[derive(Clone, Debug)]
struct ObstacleLine {
    a: Vector2<f64>,
    b: Vector2<f64>,
    bounds: Rect,
}


#[derive(Clone, Debug)]
pub struct StreetSettings {
    base_edge_len: f64,
    max_edge_len_ratio: f64,
    min_edge_len_ratio: f64,
    node_merge_dist: f64,
    // As const settings are required, they should be added here.
}


/// Trait for objects which will be added to the city map.
///
/// Allows convenient, logical additions of objects to the city map.
trait Builder {
    /// Gets nodes which will be added to the city map
    ///
    /// These simply specify points which roads may pass through.
    ///
    /// The passed node positions may may be modified by the StreetMap
    /// to combine passed positions with existing nodes on the map.
    fn build(&mut self, map: &mut StreetMap) {
        // TODO
    }
}


/// Struct handling logic for a single street segment.
struct StreetSegmentBuilder {
    a: Vector2<f64>,
    b: Vector2<f64>,
    cost: f64,  // Travel cost for this segment. Lower == better.
}


/// Struct handling creation of river segments within a city map.
///
/// This does not add an actual river, only allows one to be integrated
/// into the city map - allowing roads to follow the banks, bridges to
/// cross it, and preventing roads from being built through it.
struct CityRiverSegmentBuilder {
    a: Vector2<f64>,
    b: Vector2<f64>,
    width: f64,
}


// Implementation


impl StreetMap {

    const DEFAULT_SHAPE: Rect = Rect {
        top_left: Point { x: -3000.0f32, y: -3000.0f32 },
        bottom_right: Point { x: 3000.0f32, y: 3000.0f32 }
    };

    // Construction

    /// Produce new StreetMap.
    ///
    /// # Arguments
    /// * `settings` - StreetSettings with immutable settings which
    ///             will be kept for the lifetime of the StreetMap.
    /// 
    /// # Return
    /// StreetMap
    pub fn new(settings: StreetSettings) -> StreetMap {
        StreetMap {
            obstacles: QuadTree::default(Self::DEFAULT_SHAPE),
            nodes: QuadTree::default(Self::DEFAULT_SHAPE),
            edges: QuadTree::default(Self::DEFAULT_SHAPE),
            settings,
        }
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
    pub fn add_node(&mut self, node: &Node) -> NodeId {
        let existing = self.find_nearest_node(
            node.uv, self.settings.node_merge_dist
        );
        if existing.is_some() {
            return existing.2;
        }

        NodeId(self.nodes.insert(node.clone()))
    }

    // Accessors

    /// Get Node from NodeId
    ///
    /// # Arguments
    /// * `id` - NodeId specifying a Node
    ///
    /// # Return
    /// Node
    fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
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
    fn find_nearest_node(
        &self, uv: Vector2<f64>, r: f64
    ) -> Option<(&Node, f64, NodeId)> {
        let uv_p = vec_to_point(uv);
        let rect = Rect::centered_with_radius(&uv_p, r as f32);

        // Query Nodes within rect.
        let query_res = self.nodes.query(rect);
        if query_res.is_empty() {
            return Option::None;
        }

        // Find which result is closest.
        let first_res = query_res[0].1;
        debug_assert!(first_res.top_left() == first_res.bottom_right());
        let mut nearest_d2 = query_res[0].1.top_left().distance_2(&uv_p);
        let mut nearest_i = 0usize;
        for i in 1..query_res.len() {
            let res = query_res[i];
            let node_rect: &Rect = res.1;
            debug_assert!(node_rect.top_left() == node_rect.bottom_right());
            let d2 = node_rect.top_left().distance_2(&uv_p);
            if d2 < nearest_d2 {
                nearest_d2 = d2;
                nearest_i = i;
            }
        }

        // Check that distance to nearest node is less than r.
        // Otherwise, return None.
        let d = nearest_d2.sqrt() as f64;
        if d > r {
            return Option::None;
        }

        let nearest_res = query_res[nearest_i];
        Option::Some((nearest_res.0, d, NodeId(nearest_res.2)))
    }

    fn get_obstacle_line(&self, id: ObstacleId) -> Option<&ObstacleLine> {
        self.obstacles.get(id.0)
    }

    fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(id.0)
    }
}

impl Node {
    fn new(uv: Vector2<f64>) -> Node {
        Node {
            uv,
            edges: Vec::with_capacity(4),
        }
    }
}

impl Spatial for Node {
    fn aabb(&self) -> Rect {
        Rect::null_at(&Point { x: self.uv.x as f32, y: self.uv.y as f32 })
    }
}


impl ObstacleLine {
    fn new(a: Vector2<f64>, b: Vector2<f64>) -> ObstacleLine {
        ObstacleLine {
            a,
            b,
            bounds: find_line_bounds(a, b),
        }
    }
}


impl Spatial for ObstacleLine {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}


impl Spatial for Edge {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}


// Utility functions


/// Finds the bounding box for a Line.
///
/// # Arguments
/// * `a` - First point of line.
/// * `b` - Second point of line.
///
/// # Return
/// Bounding box Rect.
fn find_line_bounds(a: Vector2<f64>, b: Vector2<f64>) -> Rect {
    Rect::from_points(&vec_to_point(a), &vec_to_point(b))
}


fn vec_to_point(v: Vector2<f64>) -> Point {
    Point { x: v.x as f32, y: v.y as f32 }
}


// Street Map Objects


impl StreetSegmentBuilder {
    pub fn new(
        a: Vector2<f64>,
        b: Vector2<f64>,
        cost_mod: f64
    ) -> StreetSegmentBuilder {
        let cost = a.distance(b) * cost_mod;
        StreetSegmentBuilder { a, b, cost }
    }
}


#[cfg(test)]
mod tests {
    use cgmath::vec2;

    use pop::streets::{StreetSegmentBuilder, StreetMap, StreetSettings, Node};

    fn get_default_test_settings() -> StreetSettings {
        StreetSettings {
            base_edge_len: 100.0,
            max_edge_len_ratio: 1.5,
            min_edge_len_ratio: 0.5,
            node_merge_dist: 0.1,
        }
    }

    // ----------------------------
    // StreetMap

    #[test]
    fn test_simple_initialization() {
        let step = 100.0;
        for i in 0..100 {
            let u0 = i as f64 * step;
            let u1 = u0 - step;
            let v = 0.0;

            let street_segment =
                StreetSegmentBuilder::new(vec2(u0, v), vec2(u1, v), 1.0);
        }
        assert!(true);  // Placeholder
    }

    /// Test that the nearest node to a passed position can be found.
    #[test]
    fn test_find_nearest_node() {
        let settings = get_default_test_settings();
        let mut map = StreetMap::new(settings);

        map.add_node(&Node::new(vec2(0.0, 1000.0)));
        map.add_node(&Node::new(vec2(0.0, 0.0)));  // Should be nearest.
        map.add_node(&Node::new(vec2(1000.0, 0.0)));
        map.add_node(&Node::new(vec2(-500.0, -500.0)));
        map.add_node(&Node::new(vec2(100.0, -200.0)));
        map.add_node(&Node::new(vec2(-200.0, 100.0)));

        let (node, d, id) =
            map.find_nearest_node(vec2(200.0, 200.0), 300.0).unwrap();

        assert_vec2_near!(node.uv, vec2(0.0, 0.0));
    }

    /// Test that the nearest node to a passed position is not returned
    /// if the radius is too small.
    #[test]
    fn test_find_nearest_node_returns_none_if_radius_too_small() {
        let settings = get_default_test_settings();
        let mut map = StreetMap::new(settings);

        map.add_node(&Node::new(vec2(0.0, 1000.0)));
        map.add_node(&Node::new(vec2(0.0, 0.0)));  // Nearest.
        map.add_node(&Node::new(vec2(1000.0, 0.0)));

        assert!(map.find_nearest_node(vec2(200.0, 200.0), 220.0).is_none());
    }

    // ----------------------------
    // StreetSegment

}
