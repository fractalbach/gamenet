use num_traits::Float;
use cgmath::{Vector2, vec2, BaseNum};
use geo_types::{Line, LineString, Coordinate, CoordinateType, Point};

use util::point::{PointOps, VecOps};

pub trait LineOps<T: CoordinateType + BaseNum + Float> {
    fn divide(&self, n: i32) -> LineString<T>;
    fn right(&self) -> Vector2<T>;
    fn left(&self) -> Vector2<T>;
    fn dir(&self) -> Vector2<T>;
}


impl<T> LineOps<T> for Line<T>
where T: CoordinateType + BaseNum + Float
{
    /// Divide line into N segments.
    fn divide(&self, n: i32) -> LineString<T> {
        debug_assert!(n >= 1);
        let a = self.start_point().to_vec();
        let b = self.end_point().to_vec();
        let step = (b - a) / (T::from(n).unwrap());

        let mut points = vec!();
        for i in 0..(n + 1) {
            points.push((step * (T::from(i).unwrap()) + a).to_point());
        }

        LineString::from(points)
    }

    /// Gets right-side perpendicular vector.
    ///
    /// Output will -not- be normalized.
    fn right(&self) -> Vector2<T> {
        let dir = self.dir();
        vec2(dir.y, -dir.x)
    }

    /// Gets left-side perpendicular vector.
    ///
    /// Output will -not- be normalized.
    fn left(&self) -> Vector2<T> {
        let dir = self.dir();
        vec2(-dir.y, dir.x)
    }

    /// Gets direction vector from line.
    ///
    /// Will _not_ be normalized.
    fn dir(&self) -> Vector2<T> {
        self.end_point().to_vec() - self.start_point().to_vec()
    }
}


#[cfg(test)]
mod tests {
    use geo_types::{Polygon, Point, Coordinate, Line};
    use util::line::*;

    #[test]
    fn test_line_divide() {
        let original = Line::new((1.0, 1.0), (0.0, 2.0));
        let string = original.divide(4);
        assert_vec2_near!(string[0], Coordinate{x: 1.0, y: 1.0});
        assert_vec2_near!(string[1], Coordinate{x: 0.75, y: 1.25});
        assert_vec2_near!(string[2], Coordinate{x: 0.5, y: 1.5});
        assert_vec2_near!(string[3], Coordinate{x: 0.25, y: 1.75});
        assert_vec2_near!(string[4], Coordinate{x: 0., y: 2.});
    }
}
