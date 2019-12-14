
use aabb_quadtree::geom::{Rect, Point};
use cgmath::Vector2;


/// Finds the bounding box for a Line.
///
/// # Arguments
/// * `a` - First point of line.
/// * `b` - Second point of line.
///
/// # Return
/// Bounding box Rect.
pub fn find_line_bounds(a: Vector2<f64>, b: Vector2<f64>) -> Rect {
    Rect::from_points(&vec_to_point(a), &vec_to_point(b))
}


pub fn vec_to_point(v: Vector2<f64>) -> Point {
    Point { x: v.x as f32, y: v.y as f32 }
}