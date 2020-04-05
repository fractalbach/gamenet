use std::mem::swap;

use geo_types::{Polygon, Point, LineString, CoordinateType};

//
///// Splits polygon in half, using existing vertices, preferring output
///// shapes that minimize surface area (Are rounder rather than thinner)
//pub fn split_even(poly: &Polygon<f64>) -> (Polygon<f64>, Polygon<f64>) {
//    // Iterate over all combinations of polygons.
//    for (i, a) in poly.exterior().points_iter().enumerate() {
//        for b in poly.exterior().points_iter().skip(i) {
//
//        }
//    }
//}

trait PolyOps<T: CoordinateType> {

    /// Split polygon into two between two vertices.
    ///
    /// Does not support polygons that have enclaves (openings) inside.
    fn split(&self, i0: usize, i1: usize) -> (Polygon<T>, Polygon<T>);
}


impl<T> PolyOps<T> for Polygon<T> where T: CoordinateType {
    fn split(&self, mut i0: usize, mut i1: usize) -> (Polygon<T>, Polygon<T>) {
        debug_assert_ne!(i0, i1);
        debug_assert!(i1 < self.exterior().num_coords());

        if i1 < i0 {
            swap(&mut i0, &mut i1);
        }

        // Get points for first output poly.
        let mut a_points = vec!();
        let mut b_points = vec!();

        // Add first half of poly A.
        for point in self.exterior().points_iter().take(i0 + 1) {
            a_points.push(point);
        }

        // Add poly B.
        for point in self.exterior().points_iter().skip(i0).take(i1 - i0 + 1) {
            b_points.push(point);
        }

        // Add second half of poly B.
        for point in self.exterior().points_iter().skip(i1) {
            a_points.push(point);
        }

        let a = Polygon::new(LineString::from(a_points), vec!());
        let b = Polygon::new(LineString::from(b_points), vec!());

        return (a, b);
    }
}


#[cfg(test)]
mod tests {
    use geo_types::{Polygon, Point, Coordinate};
    use poly_util::*;

    macro_rules! coord {
        ($x:expr, $y:expr) => {{ Coordinate::<f64>::from(($x, $y)) }}
    }

    #[test]
    fn test_simple_split1() {
        let original = Polygon::new(
            LineString::from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.)]),
            vec![],
        );

        let (a, b) = original.split(0, 2);

        // Check A.
        assert_vec2_near!(a.exterior()[0], coord!(0., 0.));
        assert_vec2_near!(a.exterior()[1], coord!(1., 1.));
        assert_vec2_near!(a.exterior()[2], coord!(1., 0.));

        // Check B.
        assert_vec2_near!(b.exterior()[0], coord!(0., 0.));
        assert_vec2_near!(b.exterior()[1], coord!(0., 1.));
        assert_vec2_near!(b.exterior()[2], coord!(1., 1.));
    }

    #[test]
    fn test_simple_split2() {
        let original = Polygon::new(
            LineString::from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.)]),
            vec![],
        );

        let (a, b) = original.split(1, 3);

        // Check A.
        assert_vec2_near!(a.exterior()[0], coord!(0., 0.));
        assert_vec2_near!(a.exterior()[1], coord!(0., 1.));
        assert_vec2_near!(a.exterior()[2], coord!(1., 0.));

        // Check B.
        assert_vec2_near!(b.exterior()[0], coord!(0., 1.));
        assert_vec2_near!(b.exterior()[1], coord!(1., 1.));
        assert_vec2_near!(b.exterior()[2], coord!(1., 0.));
    }

    #[test]
    fn test_hex_split() {
        let original = Polygon::new(
            LineString::from(
                vec![(0., 0.), (-0.5, 0.5), (0., 1.), (1., 1.), (1.5, 0.5), (1., 0.)]
            ),
            vec![],
        );

        let (a, b) = original.split(1, 4);

        // Check A.
        assert_vec2_near!(a.exterior()[0], coord!(0., 0.));
        assert_vec2_near!(a.exterior()[1], coord!(-0.5, 0.5));
        assert_vec2_near!(a.exterior()[2], coord!(1.5, 0.5));
        assert_vec2_near!(a.exterior()[3], coord!(1., 0.));

        // Check B.
        assert_vec2_near!(b.exterior()[0], coord!(-0.5, 1.));
        assert_vec2_near!(b.exterior()[1], coord!(0., 1.));
        assert_vec2_near!(b.exterior()[2], coord!(1., 1.));
        assert_vec2_near!(b.exterior()[3], coord!(1.5, 0.5));
    }
}
