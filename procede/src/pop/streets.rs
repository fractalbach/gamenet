
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

use aabb_quadtree::QuadTree;
use aabb_quadtree::geom::{Rect, Point};
use cgmath::{Vector2, vec2};


/// Map containing town map information.
///
/// Map is composed of three basic components:
///  * Obstacles.
///  * Nodes.
///  * Settings.
struct StreetMap {
    obstacle_tree: QuadTree<ObstacleLine>,
    node_tree: QuadTree<Node>,
    settings: StreetSettings,
}


struct Node {

}


struct Edge {
    cost: f64,  // Travel cost of edge. Lower is better.
    a: usize,
    b: usize
}


struct ObstacleLine {
    a: Vector2<f64>,
    b: Vector2<f64>,
    bounds: Rect,
}


struct StreetSettings {
    // Empty for now.
    // As const settings are required, they should be added here.
}


/// Trait for objects which will be added to the city map.
///
/// Allows convenient, logical additions of objects to the city map.
trait StreetMapObj {
    /// Gets obstacle lines which will be added to city map.
    fn get_obstacle_lines(&self) -> Vec<ObstacleLine>;

    /// Gets nodes which will be added to the city map
    ///
    /// These simply specify points which roads may pass through.
    ///
    /// The passed node positions may may be modified by the StreetMap
    /// to combine passed positions with existing nodes on the map.
    fn get_nodes(&self) -> Vec<Node>;

    /// Gets street edges (connections between nodes) which will be
    /// added to the map.
    fn get_edges(&self) -> Vec<Edge>;
}


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
            obstacle_tree: Self::make_obstacle_tree(&settings),
            node_tree: Self::make_node_tree(&settings),
            settings,
        }
    }


    fn make_obstacle_tree(settings: &StreetSettings) -> QuadTree<ObstacleLine> {
        let mut map: QuadTree<ObstacleLine> =
            QuadTree::default(Self::DEFAULT_SHAPE);
        // TODO
        map
    }

    fn make_node_tree(settings: &StreetSettings) -> QuadTree<Node> {
        let mut map: QuadTree<Node> = QuadTree::default(Self::DEFAULT_SHAPE);
        // TODO
        map
    }

    // Addition methods.

    pub fn add(&mut self, obj: &StreetMapObj) {
        for obstacle in &obj.get_obstacle_lines() {
            self.add_obstacle_line(obstacle);
        }
        for node in &obj.get_nodes() {
            self.add_node(node);
        }
        for edge in &obj.get_edges() {
            self.add_edge(edge);
        }
    }

    pub fn add_all<'a, I>(&mut self, objects: I)
    where I: Iterator<Item = &'a StreetMapObj> {
        for obj in objects {
            self.add(obj);
        }
    }

    fn add_obstacle_line(&mut self, line: &ObstacleLine) -> usize {
        0  // TODO
    }

    fn add_node(&mut self, node: &Node) -> usize {
        0 // TODO
    }

    fn add_edge(&mut self, edge: &Edge) -> usize {
        0 // TODO
    }
}


impl Node {

}


impl ObstacleLine {
    fn new(a: Vector2<f64>, b: Vector2<f64>) -> ObstacleLine {
        ObstacleLine {
            a,
            b,
            bounds: Self::find_bounds(a, b),
        }
    }

    /// Finds the bounding box for an ObstacleLine.
    ///
    /// # Arguments
    /// * `a` - First point of ObstacleLine.
    /// * `b` - Second point of ObstacleLine.
    ///
    /// # Return
    /// Bounding box Rect.
    fn find_bounds(a: Vector2<f64>, b: Vector2<f64>) -> Rect {
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
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_street_placeholder() {
        assert!(true);  // Placeholder
    }

}
