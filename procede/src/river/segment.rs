use aabb_quadtree::geom::{Rect, Point};
use aabb_quadtree::Spatial;
use cgmath::{Basis2, Vector2, Rad, Rotation, Rotation2, vec2};
use cgmath::InnerSpace;

use river::common::get_base_width;
use river::river_graph::Node;
use river::curve::{Curve, ProjectionInfo};
pub use river::curve::RiverSide;


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
    slope: f64,
    upriver_strahler: i8,
    downriver_strahler: i8,
}

/// Struct containing data about a position relative to a nearby
/// river segment.
pub struct NearSegmentInfo {
    pub side: RiverSide,  // TODO: enum
    pub dist: f64,
    pub dist_widths: f64,
    pub w: f64,
    pub depth: f64,
    pub upriver_strahler: i8,
    pub fp_strahler: f64,
    pub band_w: f64,
}


impl Segment {
    const MAX_STRAHLER: i8 = 12;
    const MAX_MEANDER_BAND: f64 = get_base_width(Self::MAX_STRAHLER) * 20.0;
    const MIN_BAND_WIDTH_SOPE: f64 = 0.1;
    const BASE_BOUND_MARGIN: f64 = Self::MAX_MEANDER_BAND * 2.0;
    const STRAHLER_INC_W_RATIO: f64 = 0.7;
    const CONTROL_NODE_DIST_RATIO: f64 = 0.25;

    pub fn new(downriver: &Node, upriver: &Node) -> Segment {
        let base_curve = Curve::new(
            upriver.uv,
            Self::upriver_control_node(downriver, upriver),
            Self::downriver_control_node(downriver, upriver),
            downriver.uv
        );
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
            upriver_strahler: upriver.strahler,
            downriver_strahler: downriver.strahler,
            slope: Self::find_slope(downriver, upriver),
        }
    }

    // Constructor helpers.

    /// Finds downriver control node position.
    ///
    /// The down-river control node is the control node closer to
    /// the downriver of the endpoints of a segment.
    ///
    /// # Arguments
    /// * `downriver` - Reference to downriver node.
    /// * `upriver` - Reference to upriver node.
    ///
    /// # Return
    /// UV Position of the downriver control node.
    fn downriver_control_node(
        downriver: &Node, upriver: &Node
    ) -> Vector2<f64> {
        // Determine distance of control node from downriver node.
        let end_node_separation = (upriver.uv - downriver.uv).magnitude();
        let distance = Self::CONTROL_NODE_DIST_RATIO * end_node_separation;

        // Determine direction from downriver end node.
        let direction;
        if downriver.is_fork() {
            let rotation: Basis2<f64> = if downriver.left_inlet() == upriver.i {
                Rotation2::from_angle(Rad(downriver.fork_angle / -2.0))
            } else {
                Rotation2::from_angle(Rad(downriver.fork_angle / 2.0))
            };
            direction = rotation.rotate_vector(-downriver.direction);
        } else {
            direction = downriver.direction * -1.0;
        }

        // Determine node position.
        // Direction points downriver, so the reciprocal is used.
        let pos = downriver.uv + direction * distance;
        pos
    }

    /// Finds up-river control node position.
    ///
    /// The up-river control node is the control node closer to
    /// the upriver node of a segment.
    ///
    /// # Arguments
    /// * `downriver` - Reference to downriver node.
    /// * `upriver` - Reference to upriver node.
    ///
    /// # Return
    /// UV Position of the up-river control node.
    fn upriver_control_node(
        downriver: &Node, upriver: &Node
    ) -> Vector2<f64> {
        // Determine distance of control node from downriver node.
        let end_node_separation = (upriver.uv - downriver.uv).magnitude();
        let distance = Self::CONTROL_NODE_DIST_RATIO * end_node_separation;

        // Determine node position.
        let pos = upriver.uv + upriver.direction * distance;
        pos
    }

    /// Finds the bounding box for a river segment from its base curve.
    ///
    /// # Arguments
    /// * `curve` - Base curve of the Segment.
    /// * `margin` - Margin around curve which is added to bounding box.
    ///
    /// # Return
    /// Bounding box Rect.
    fn find_bounds(curve: &Curve, margin: f64) -> Rect {
        let mut min_x: f64 = curve.a().x;
        let mut max_x: f64 = curve.a().x;
        let mut min_y: f64 = curve.a().y;
        let mut max_y: f64 = curve.a().y;

        for point in &[curve.b(), curve.ctrl_b(), curve.ctrl_a()] {
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

    /// Finds slope of river segment.
    ///
    /// The returned slope is approximate. It treats the segment as a
    /// straight line between the upriver and downriver nodes.
    ///
    /// # Arguments
    /// * `downriver` - Node at downriver end of segment.
    /// * `upriver` - Node at upriver end of segment.
    ///
    /// # Return
    /// The slope of the river segment as a ratio between 0.0 and -1.0.
    fn find_slope(downriver: &Node, upriver: &Node) -> f64 {
        let diff = downriver.h - upriver.h;
        debug_assert!(diff <= 0.0);
        let dist = (downriver.uv - upriver.uv).magnitude();
        diff / dist
    }

    // Instance methods.

    /// Finds river info determined by the segment at a given position.
    ///
    /// # Arguments
    /// * `uv` - Position in UV-space.
    ///
    /// # Return
    /// RiverInfo determined by Segment.
    fn info(&self, uv: Vector2<f64>) -> NearSegmentInfo {
        let curve_info = self.base_curve.project(uv);
        let d = curve_info.distance;
        let w = self.base_width(curve_info.t);
        let dist_widths = d / w;
        let depth = if d < w { (w - d).sqrt() } else { 0.0 };

        NearSegmentInfo {
            side: curve_info.side,
            dist: curve_info.distance,
            dist_widths,
            w,
            depth,
            upriver_strahler: self.upriver_strahler,
            fp_strahler: self.find_fp_strahler(curve_info.t),
            band_w: self.find_band_width(w, self.slope),
        }
    }

    /// Finds base river width at passed ratio of length.
    ///
    /// # Arguments
    /// * `t` - Ratio of distance upriver.
    ///             0.0 == downriver end, 1.0 == upriver end.
    ///
    /// # Return
    /// Base width of river at identified point.
    fn base_width(&self, t: f64) -> f64 {
        self.downriver_w * t + self.upriver_w * (1.0 - t)
    }
    
    /// Finds river depth at a uv position
    ///
    /// # Arguments
    /// * `curve_info` - Curve info from the point at which depth is desired
    ///
    /// # Return
    /// River water depth in meters at passed position.
    fn find_depth(&self, curve_info: ProjectionInfo) -> f64 {
        let w = self.base_width(curve_info.t);
        let d = curve_info.distance;
        let depth;
        if d >= w {
            depth = 0.0;
        } else {
            depth = (w - d).sqrt();
        }
        depth
    }
    
    /// Finds floating point strahler value at a sample point.
    ///
    /// A true strahler number is only ever an integer, however some
    /// algorithms used by the river module require smooth transitions
    /// along a river segment, and therefore a floating point
    /// interpolation of the strahler number is provided.
    ///
    /// This method returns an interpolation between the strahler
    /// number at the segment's downriver and upriver ends.
    ///
    /// # Arguments
    /// * `t` - Ratio of distance upriver.
    ///             0.0 == downriver end, 1.0 == upriver end.
    ///
    /// # Return
    /// Strahler number interpolation.
    fn find_fp_strahler(&self, t: f64) -> f64 {
        self.upriver_strahler as f64 * t + 
                self.downriver_strahler as f64 * (1.0 - t)
    }
    
    /// Finds the river band width at a sample point.
    ///
    /// The river band is the region within which a river meanders,
    /// and is usually visible as a flat area around the river with
    /// soft, heavily vegitated ground, and little or no construction.
    ///
    /// # Arguments
    /// * `w` - Width at passed sample point
    /// * `slope` - Approximate slope of the Segment.
    ///
    /// # Return
    /// River band width in meters.
    fn find_band_width(&self, w: f64, slope: f64) -> f64 {
        if slope <= Self::MIN_BAND_WIDTH_SOPE {
            return w;
        }
        let max_slope_ratio = slope / Self::MIN_BAND_WIDTH_SOPE;
        let band_ratio = (1.0 - max_slope_ratio) * Self::MAX_MEANDER_BAND;
        band_ratio * w
    }
}

impl Spatial for Segment {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}


// --------------------------------------------------------------------
