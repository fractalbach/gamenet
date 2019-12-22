use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;
use cgmath::MetricSpace;

use pop::streets::map::{
    TownMap, Node, NodeId, Edge, EdgeId, ObstacleLine, ObstacleId
};


/// Trait for objects which will be added to the city map.
///
/// Allows convenient, logical additions of objects to the city map.
pub trait Builder {
    /// Gets nodes which will be added to the city map
    ///
    /// These simply specify points which roads may pass through.
    ///
    /// The passed node positions may may be modified by the StreetMap
    /// to combine passed positions with existing nodes on the map.
    fn build(&mut self, map: &mut TownMap);
}


/// Struct handling logic for a single street segment.
pub struct StreetSegmentBuilder {
    a: Vector2<f64>,
    b: Vector2<f64>,
    cost: f64,  // Travel cost for this segment. Lower == better.
}


/// Struct handling creation of river segments within a city map.
///
/// This does not add an actual river, only allows one to be integrated
/// into the city map - allowing roads to follow the banks, bridges to
/// cross it, and preventing roads from being built through it.
pub struct CityRiverSegmentBuilder {
    a: Vector2<f64>,
    b: Vector2<f64>,
    width: f64,
}


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

impl Builder for StreetSegmentBuilder {
    fn build(&mut self, map: &mut TownMap) {
        let a = map.add_node(Node::new(self.a));
        let b = map.add_node(Node::new(self.b));
        map.add_edge_between(a, b, self.cost);
        map.add_obstacle(ObstacleLine::new(map.node(a).uv(), map.node(b).uv()));
    }
}
