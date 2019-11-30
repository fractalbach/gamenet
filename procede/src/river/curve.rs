use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;
use lyon_geom::CubicBezierSegment;
use lyon_geom::euclid::{Point2D, Vector2D};
use lyon_geom::euclid::default::Point2D as Point;

/// A single river bezier curve.
///
/// Handles calculation of a point's distance to a curve.
pub struct Curve {
    segment: CubicBezierSegment<f64>,
}

/// Struct containing the results of a projection.
pub struct ProjectionInfo {
    pub point: Vector2<f64>,
    pub distance: f64,
    pub side: i8,
    pub t: f64,
}

impl Curve {
    /// Construct new Curve from end points and control nodes.
    pub fn new(
        a: Vector2<f64>,
        ctrl_a: Vector2<f64>,
        ctrl_b: Vector2<f64>,
        b: Vector2<f64>
    ) -> Curve {
        use lyon_geom::euclid::Point2D;
        Curve {
            segment: CubicBezierSegment {
                from: Point2D::new(a.x, a.y),
                ctrl1: Point2D::new(ctrl_a.x, ctrl_a.y),
                ctrl2: Point2D::new(ctrl_b.x, ctrl_b.y),
                to: Point2D::new(b.x, b.y),
            }
        }
    }

    /// Find the projection of a point onto the Curve.
    ///
    /// This is the point on the Curve that is closest to the
    /// passed point.
    pub fn project(&self, p: Vector2<f64>) -> ProjectionInfo {
        // The closest point on the Curve to the passed point is
        // also the point where the derivative of the curve is
        // perpendicular to the vector from the passed point to the
        // point on the curve.
        //
        // This can be exploited to perform a binary search of
        // the Curve.

        const PRIMARY_POINTS: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

        // Convert input to euclid vector.
        let p = Point2D::new(p.x, p.y);

        // Find derivatives at primary points on the Curve and get the
        // cosine of the angle between the derivative vector and the
        // direction vector from p to the primary point.
        let mut primary_cos_theta = [0.0; 5];
        for (i, &t) in PRIMARY_POINTS.iter().enumerate() {
            primary_cos_theta[i] = self.find_cos_theta(p, t);
        }

        // Find nearest
        let mut nearest_p = Point2D::zero();
        let mut nearest_d2 = -1.0;
        let mut nearest_t = 0.0;

        // If sample at passed t is nearest point found, set nearest
        // values to the samples associated with passed t.
        let mut make_nearest_or_ignore = |t: f64| {
            let sample_p = self.segment.sample(t);
            let sample_d2 = (sample_p - p).square_length();
            if nearest_d2 < 0.0 || sample_d2 < nearest_d2 {
                nearest_p = sample_p;
                nearest_d2 = sample_d2;
                nearest_t = t;
            }
        };

        // If derivative at first point is facing -away- from p, then
        // it is a candidate to be the closest point.
        if Self::is_opening(primary_cos_theta[0]) {
            make_nearest_or_ignore(0.0);
        }
        // If derivative at last point is facing -toward- p, then
        // it is a candidate to be the closest point.
        if Self::is_closing(primary_cos_theta[4]) {
            make_nearest_or_ignore(1.0);
        }
        // If between any two points the curve derivative changes from
        // facing -toward- p to -away- from p, then the point where
        // the cosine passes through 0.0 is a candidate to be the
        // closest point.
        for i in 0..4 {
            if
                Self::is_closing(primary_cos_theta[i]) &&
                Self::is_opening(primary_cos_theta[i + 1])
            {
                let t0 = PRIMARY_POINTS[i];
                let t1 = PRIMARY_POINTS[i + 1];
                let perpendicular_t = self.binary_search(p, t0, t1);
                make_nearest_or_ignore(perpendicular_t);
            }
        }

        ProjectionInfo {
            point: vec2(nearest_p.x, nearest_p.y),
            distance: nearest_d2.sqrt(),
            side: self.find_side(p, nearest_t),
            t: nearest_t,
        }
    }

    // Getters

    pub fn a(&self) -> Vector2<f64> {
        vec2(self.segment.from.x, self.segment.from.y)
    }

    pub fn ctrl_a(&self) -> Vector2<f64> {
        vec2(self.segment.ctrl1.x, self.segment.ctrl1.y)
    }

    pub fn ctrl_b(&self) -> Vector2<f64> {
        vec2(self.segment.ctrl2.x, self.segment.ctrl2.y)
    }

    pub fn b(&self) -> Vector2<f64> {
        vec2(self.segment.to.x, self.segment.to.y)
    }
    
    pub fn sample(&self, t: f64) -> Vector2<f64> {
        let p = self.segment.sample(t);
        vec2(p.x, p.y)
    }
    
    pub fn derivative(&self, t: f64) -> Vector2<f64> {
        let d = self.segment.derivative(t);
        vec2(d.x, d.y)
    }

    // Private helpers

    /// Given a cosign of the angle between the direction to the sample
    /// point and the derivative of the bezier, returns whether or not
    /// the derivative is approaching the uv pos.
    fn is_closing(cos_theta: f64) -> bool { cos_theta < 0.0 }

    /// Given a cosign of the angle between the direction to the sample
    /// point and the derivative of the beizer, returns whether or not
    /// the derivative is diverging from the uv pos.
    fn is_opening(cos_theta: f64) -> bool { cos_theta >= 0.0 }

    /// Finds direction vector from p to the sample of t, the derivative
    /// of the curve at t, and returns the cosign of the angle between
    /// the two vectors.
    ///
    /// This value will be zero if the two vectors are perpendicular,
    /// 1.0 if the vectors are parallel, and -1 if the two vectors are
    /// directly opposite.
    fn find_cos_theta(&self, p: Point<f64>, t: f64) -> f64 {
        let curve_point = self.segment.sample(t);
        let curve_derivative = self.segment.derivative(t).normalize();
        let projection_vec = (curve_point - p).normalize();
        let cos_theta = curve_derivative.dot(projection_vec);
        cos_theta
    }
    
    // Binary search may be replaced by a more efficient
    // search method if one is found.

    /// Finds the nearest point.
    ///
    /// Expects to be passed a segment of the Curve which begins with a
    /// derivative that is approaching point p, and ends diverging from
    /// point p.
    fn binary_search(
        &self, p: Point<f64>, mut t0: f64, mut t1: f64
    ) -> f64 {
        debug_assert!(Self::is_closing(self.find_cos_theta(p, t0)));
        debug_assert!(Self::is_opening(self.find_cos_theta(p, t1)));

        for _ in 0..16 {
            let mid = (t0 + t1) / 2.0;
            if Self::is_closing(self.find_cos_theta(p, mid)) {
                t0 = mid;
            } else {
                t1 = mid;
            }
        }

        (t0 + t1) / 2.0
    }

    /// Finds which side of the curve a point is on.
    fn find_side(&self, p: Point<f64>, projection_t: f64) -> i8 {
        let derivative = self.segment.derivative(projection_t);
        -1  // TODO
    }
}

// --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use cgmath::{Vector2, vec2};

    use river::curve::*;

    /// Exhaustively searches curve for the projection of a point.
    fn exhaustive_search(curve: &Curve, p: Vector2<f64>) -> Vector2<f64> {
        let mut nearest = vec2(0.0, 0.0);
        let mut nearest_d = -1.0;
        let n_samples = 2000;
        for i in 0..n_samples {
            let t = i as f64 / n_samples as f64;
            let sample = curve.sample(t);
            let sample_d = (sample - p).magnitude();
            if nearest_d < 0.0 || sample_d < nearest_d {
                nearest = sample;
                nearest_d = sample_d;
            }
        }
        nearest
    }

    #[test]
    fn test_projection_at_convex_projection_point() {
        // Tests curve projection method against an exhaustive search.
        let curve = Curve::new(
            vec2(0.0, 0.0),
            vec2(2000.0, 1000.0),
            vec2(8000.0, -1000.0),
            vec2(10_000.0, 0.0),
        );
        
        let p = vec2(2000.0, 4000.0);
        let smart = curve.project(p).point;
        let exhaustive = exhaustive_search(&curve, p);

        assert_vec2_near!(smart, exhaustive, 10.0);
    }

    #[test]
    fn test_projection_at_concave_projection_point() {
        // Tests curve projection method against an exhaustive search.
        let curve = Curve::new(
            vec2(0.0, 0.0),
            vec2(2000.0, 1000.0),
            vec2(8000.0, -1000.0),
            vec2(10_000.0, 0.0),
        );

        let p = vec2(8000.0, 4000.0);
        let smart = curve.project(p).point;
        let exhaustive = exhaustive_search(&curve, p);

        assert_vec2_near!(smart, exhaustive, 10.0);
    }

    #[test]
    fn test_projection_past_downriver_end() {
        // Tests curve projection method against an exhaustive search.
        let curve = Curve::new(
            vec2(0.0, 0.0),
            vec2(2000.0, 1000.0),
            vec2(8000.0, -1000.0),
            vec2(10_000.0, 0.0),
        );

        let p = vec2(-2000.0, 0.0);
        let smart = curve.project(p).point;
        let exhaustive = exhaustive_search(&curve, p);

        assert_vec2_near!(smart, exhaustive, 10.0);
    }

    #[test]
    fn test_projection_past_upriver_end() {
        // Tests curve projection method against an exhaustive search.
        let curve = Curve::new(
            vec2(0.0, 0.0),
            vec2(2000.0, 1000.0),
            vec2(8000.0, -1000.0),
            vec2(10_000.0, 0.0),
        );

        let p = vec2(12000.0, 0.0);
        let smart = curve.project(p).point;
        let exhaustive = exhaustive_search(&curve, p);

        assert_vec2_near!(smart, exhaustive, 10.0);
    }
}
