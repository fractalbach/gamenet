use aabb_quadtree::geom::{Rect, Point};
use aabb_quadtree::Spatial;
use cgmath::Vector2;

use river::common::{RiverInfo, get_base_width};
use river::river_graph::Node;


/// River segment joining two nodes.
///
/// Handles calculation of river info based on distance to the
/// Segment's river course.
///
/// The Segment will handle any blending of data from different curves,
pub struct Segment {
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


impl Segment {
    const MAX_STRAHLER: i8 = 12;
    const MAX_MEANDER_BAND: f64 = get_base_width(Self::MAX_STRAHLER) * 20.0;
    const BASE_BOUND_MARGIN: f64 = Self::MAX_MEANDER_BAND * 2.0;
    const STRAHLER_INC_W_RATIO: f64 = 0.7;

    pub fn new(downriver: &Node, upriver: &Node) -> Segment {
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
