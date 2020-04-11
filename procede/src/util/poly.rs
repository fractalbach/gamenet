use num_traits::{Float, FromPrimitive};
use std::f64;
use std::mem::swap;
use std::usize;

use geo_types::{Polygon, Point, LineString, CoordinateType};
use geo::algorithm::euclidean_length::EuclideanLength;
use geo::algorithm::area::Area;

use quad::{Rect, Spatial};
use util::point::PointOps;


/// Additional utility operations for Polygon.
pub trait PolyOps<T: CoordinateType> {
    /// Creates a new Polygon with a passed exterior.
    fn from_exterior(exterior: Vec<Point<T>>) -> Polygon<T>;

    /// Split polygon into two between two vertices.
    ///
    /// Does not support polygons that have interior edges (gaps).
    fn split(&self, i0: usize, i1: usize) -> (Polygon<T>, Polygon<T>);


    /// Split polygon into two, preferring pieces that are rounder
    /// rather than thinner.
    ///
    /// Does not support polygons that have interior edges (gaps).
    ///
    /// By default, limits sample points to 32.
    fn halve(&self) -> (Polygon<T>, Polygon<T>);

    /// Split polygon into two, preferring pieces that are rounder
    /// rather than thinner.
    ///
    /// Does not support polygons that have interior edges (gaps).
    ///
    /// Python function for calculating number of iterations that will be
    /// performed for a given number of samples:
    /// ```
    /// def n_iter(n: int) -> int:
    ///     i = 0
    ///     for j in range(n):
    ///         for k in range(j + 1, n):
    ///             i += 1
    ///     return i
    /// ```
    fn halve_with_samples(&self, samples: usize) -> (Polygon<T>, Polygon<T>);

    /// Gets exterior perimeter length.
    fn perimeter(&self) -> T;
}


impl<T> PolyOps<T> for Polygon<T>
where
    T: CoordinateType + Float + FromPrimitive,
    geo::LineString<T>: geo::prelude::EuclideanLength<T>,
    geo::Polygon<T>: geo::prelude::Area<T>,
{
    fn from_exterior(exterior: Vec<Point<T>>) -> Polygon<T> {
        Polygon::new(
            LineString::from(exterior),
            vec!(),
        )
    }

    fn split(&self, mut i0: usize, mut i1: usize) -> (Polygon<T>, Polygon<T>) {
        debug_assert_ne!(i0, i1);
        debug_assert!(i1 < self.exterior().num_coords());
        debug_assert_eq!(self.interiors().len(), 0);

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

    fn halve(&self) -> (Polygon<T>, Polygon<T>) {
        // A maximum of 32 points are sampled. (496 iterations).
        return self.halve_with_samples(32);
    }

    fn halve_with_samples(&self, max_samples: usize) -> (Polygon<T>, Polygon<T>) {
        let mut min_result: T = T::from_f64(f64::INFINITY).unwrap();
        let mut min_i0: usize = usize::MAX;
        let mut min_i1: usize = usize::MAX;
        // Iterate over all unique pairings of points, finding the
        // pairing that has the smallest perimeter to area ratio squared
        // for both resulting polygons.
        let n_points = self.exterior().num_coords();
        let decimation = 1 + n_points / max_samples;
        for i0 in (0..n_points).step_by(decimation) {
            for i1 in ((i0 + 1)..n_points).step_by(decimation) {
                let (poly_a, poly_b) = self.split(i0, i1);
                let ratio_a: T = poly_a.perimeter() / poly_a.area();
                let ratio_b: T = poly_b.perimeter() / poly_b.area();
                let ratio_a_sq = ratio_a * ratio_a;
                let ratio_b_sq = ratio_b * ratio_b;
                let result = ratio_a_sq + ratio_b_sq;
                if result < min_result {
                    min_result = result;
                    min_i0 = i0;
                    min_i1 = i1;
                }
            }
        }

        debug_assert!(min_i0 != usize::MAX);
        debug_assert!(min_i1 != usize::MAX);
        return self.split(min_i0, min_i1);
    }

    fn perimeter(&self) -> T {
        return self.exterior().euclidean_length();
    }
}


impl Spatial for Polygon<f64> {
    fn aabb(&self) -> Rect {
        let to_vec = |p: Point<f64>| p.to_vec();
        let mut rect = Rect::null_at(
            self.exterior().points_iter().map(to_vec).nth(0).unwrap()
        );
        let points = self.exterior().points_iter().map(to_vec);
        for p in points.skip(1) {
            rect.expand_to_include(p);
        }
        rect
    }
}


#[cfg(test)]
mod tests {
    use geo_types::{Polygon, Point, Coordinate};
    use util::poly::*;

    use assert_approx_eq::assert_approx_eq;

    macro_rules! coord {
        ($x:expr, $y:expr) => {{ Coordinate::<f64>::from(($x, $y)) }}
    }

    /// Test shape resembles:
    ///
    ///    o     o
    ///
    ///    o     o
    ///
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

    /// Test shape resembles:
    ///
    ///    o     o
    ///
    ///    o     o
    ///
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

    /// Test shape resembles:
    ///
    ///    o     o
    /// o           o
    ///    o     o
    ///
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

    /// Test shape resembles:
    ///
    ///    o     o
    ///
    ///    o     o
    ///
    #[test]
    fn test_perimeter() {
        let poly: Polygon<f64> = Polygon::new(
            LineString::from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.)]),
            vec![],
        );
        assert_approx_eq!(poly.perimeter(), 4.);
    }

    /// Tests that poly is broken in halves predictably.
    ///
    /// Test shape resembles:
    ///
    ///    o  o  o
    /// o           o
    ///    o  o  o
    #[test]
    fn test_halve() {
        let original = Polygon::new(
            LineString::from(
                vec![
                    (0., 0.), (-0.5, 0.5), (0., 1.), (0.5, 1.),
                    (1., 1.), (1.5, 0.5), (1., 0.), (0.5, 0.),
                ]
            ),
            vec![],
        );

        let (a, b) = original.halve();

        // Check A.
        assert_vec2_near!(a.exterior()[0], coord!(0., 0.));
        assert_vec2_near!(a.exterior()[1], coord!(-0.5, 0.5));
        assert_vec2_near!(a.exterior()[2], coord!(0., 1.));
        assert_vec2_near!(a.exterior()[3], coord!(0.5, 1.));
        assert_vec2_near!(a.exterior()[4], coord!(0.5, 0.));

        // Check B.
        assert_vec2_near!(b.exterior()[0], coord!(0.5, 1.));
        assert_vec2_near!(b.exterior()[1], coord!(1., 1.));
        assert_vec2_near!(b.exterior()[2], coord!(1.5, 0.5));
        assert_vec2_near!(b.exterior()[3], coord!(1., 0.));
        assert_vec2_near!(b.exterior()[4], coord!(0.5, 0.));
    }
}
