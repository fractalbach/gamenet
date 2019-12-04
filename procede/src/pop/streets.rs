
//! Todo:
//!
//! ! Add Navigable trait.
//! Implement Navigable types:
//!     StreetSegment
//!     CityRiverSegment
//!     BuildingSegment
//! Implement TownMap.add(o: Navigable)
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
    a: u32,
    b: u32,
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


pub struct StreetSettings {
    ideal_edge_len: f64,
    max_edge_len: f64,
    min_edge_len: f64
    // As const settings are required, they should be added here.
}


/// Trait for objects which will be added to the city map.
///
/// Allows convenient, logical additions of objects to the city map.
trait StreetMapObj {
    /// Gets nodes which will be added to the city map
    ///
    /// These simply specify points which roads may pass through.
    ///
    /// The passed node positions may may be modified by the StreetMap
    /// to combine passed positions with existing nodes on the map.
    fn add(&mut self, map: &mut StreetMap) {
        // TODO
    }
}


/// Struct handling logic for a single street segment.
struct StreetSegment {
    a: Vector2<f64>,
    b: Vector2<f64>,
    cost: f64,  // Travel cost for this segment. Lower == better.
}


struct CityRiverSegment {
    a: Vector2<f64>,
    b: Vector2<f64>,
}


// Implementation


impl StreetMap {

    const DEFAULT_SHAPE: Rect = Rect {
        top_left: Point { x: -3000.0f32, y: -3000.0f32 },
        bottom_right: Point { x: 3000.0f32, y: 3000.0f32 }
    };

    // Construction

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
    pub fn add(&mut self, obj: &StreetMapObj) {
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
    /// `node` - Node reference to be added.
    ///
    /// # Return
    /// NodeId pointing to added node, or existing nearby node which
    /// should be used instead.
    fn add_node(&mut self, node: &Node) -> NodeId {
        NodeId(self.nodes.insert(node.clone()))
    }

    // Accessors

    fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    fn get_obstacle_line(&self, id: ObstacleId) -> Option<&ObstacleLine> {
        self.obstacles.get(id.0)
    }

    fn get_edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(id.0)
    }
}

// TODO: Add Object create() and Description implementations.

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
/// * `a` - First point of ObstacleLine.
/// * `b` - Second point of ObstacleLine.
///
/// # Return
/// Bounding box Rect.
fn find_line_bounds(a: Vector2<f64>, b: Vector2<f64>) -> Rect {
    macro_rules! min {
            ($a:expr, $b:expr) => {{
                let (a, b) = ($a, $b);
                if a < b { a } else { b }
            }};
        };
    macro_rules! max {
            ($a:expr, $b:expr) => {{
                let (a, b) = ($a, $b);
                if a < b { b } else { a }
            }};
        };

    let min_x = min!(a.x, b.x);
    let max_x = max!(a.x, b.x);
    let min_y = min!(a.y, b.y);
    let max_y = max!(a.y, b.y);

    // Create Rect. Note that the top-left field contains the
    // minimums, due to the quad-tree library being intended for
    // 2d graphics applications.
    Rect {
        top_left: Point { x: min_x as f32, y: min_y as f32 },
        bottom_right: Point { x: max_x as f32, y: max_y as f32 }
    }
}


// Street Map Objects


impl StreetSegment {
    pub fn new(
        a: Vector2<f64>,
        b: Vector2<f64>,
        cost_mod: f64
    ) -> StreetSegment {
        let cost = a.distance(b) * cost_mod;
        StreetSegment { a, b, cost }
    }
}


#[cfg(test)]
mod tests {
    use cgmath::vec2;

    use pop::streets::StreetSegment;

    #[test]
    fn test_simple_initialization() {
        let step = 100.0;
        for i in 0..100 {
            let u0 = i as f64 * step;
            let u1 = u0 - step;
            let v = 0.0;

            let street_segment =
                StreetSegment::new(vec2(u0, v), vec2(u1, v), 1.0);
        }
        assert!(true);  // Placeholder
    }
}
