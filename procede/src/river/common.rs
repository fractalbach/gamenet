use aabb_quadtree::geom::Point;
use cgmath::Vector2;

/// Struct used to return height and related information about
/// a position from the RiverLayer.
pub struct RiverInfo {
    pub height: f64,
}


pub const fn get_base_width(strahler: i8) -> f64 {
    // Width table based on real-world measurements.
    const LOOKUP: [f64; 13] = [
        1.0,  // 0
        1.5,  // 1
        2.0,  // 2
        5.0,  // 3
        10.0,  // 4
        50.0,  // 5
        100.0,  // 6
        180.0,  // 7
        400.0,  // 8
        800.0,  // 9
        1000.0,  // 10
        2000.0,  // 11
        4000.0,  // 12
    ];

    LOOKUP[strahler as usize]
}
// --------------------------------------------------------------------


/// Converts Vector2 to Point for use in QuadTree.
///
/// As the precision is lowered from f64 to f32, some information
/// will be lost in the conversion.
///
/// # Arguments
/// * `v` - Vector2 to be converted to a Point.
///
/// # Return
/// Point
pub fn vec2pt(v: Vector2<f64>) -> Point {
    Point {
        x: v.x as f32,
        y: v.y as f32,
    }
}


#[cfg(test)]
mod tests {
    use river::common::*;

    #[test]
    fn test_base_width() {
        assert_in_range!(0.75, get_base_width(0), 1.5);
        assert_in_range!(1.0, get_base_width(1), 2.0);
        assert_in_range!(5.0, get_base_width(4), 50.0);
        assert_in_range!(700.0, get_base_width(10), 2000.0);
        assert_in_range!(3000.0, get_base_width(12), 8000.0);
    }
}
