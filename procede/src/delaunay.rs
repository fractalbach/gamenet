//! Contains adapter for the voronoi module.
use std::iter::FromIterator;

use geo_types::{Polygon, Line, Point, LineString};
use delaunay2d::Delaunay2D;

use quad::Rect;
use util::vec2::{VecOps, ToVec2};
use itertools::max;
use util::PolyOps;
use util::vec::VecMap;


pub struct Delaunay(Delaunay2D);


impl Delaunay {
    pub fn new(radius: f64) -> Delaunay {
        Delaunay(Delaunay2D::new((0., 0.), radius))
    }

    pub fn with_center(radius: f64, center: Point<f64>) -> Delaunay {
        Delaunay(Delaunay2D::new((center.x(), center.y()), radius))
    }

    pub fn from_points(points: Vec<Point<f64>>, margin: f64) -> Delaunay {
        let point_vectors = points.map(|point| {
            point.to_vec()
        });
        let rect = Rect::from_point_vec(point_vectors).expand_by(margin);
        let center = rect.midpoint();
        let corner = rect.top_left() - rect.midpoint();
        let radius = (corner.x.powi(2) + corner.y.powi(2)).sqrt();

        let mut delaunay = Delaunay::with_center(radius, center.to_point());
        for point in points {
            delaunay.add_point(point);
        }
        delaunay
    }

    pub fn add_point(&mut self, point: Point<f64>) {
        self.0.add_point((point.x(), point.y()))
    }

    pub fn voronoi_regions(&self) -> (Vec<Point<f64>>, Vec<Vec<usize>>) {
        let (points, regions) = self.0.export_voronoi_regions();
        let points = points.map(
            |&(x, y)| Point::new(x, y)
        );
        (points, regions)
    }

    pub fn voronoi_polygons(&self) -> Vec<Polygon<f64>> {
        let (points, regions) = self.voronoi_regions();
        regions.map(|region| {
            Polygon::from_exterior(region.map(|&i| {
                points[i]
            }))
        })
    }
}
