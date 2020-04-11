//! Contains adapter for the voronoi module.
use std::iter::FromIterator;

use geo_types::{Polygon, Line, Point, LineString};
pub use voronoi::{voronoi, DCEL, Point as VoronoiPoint};
use voronoi::make_polygons as make_polygons_inner;
use voronoi::make_line_segments as make_line_segments_inner;


pub fn make_polygons(dcel: &DCEL) -> Vec<Polygon<f64>> {
    Vec::from_iter(make_polygons_inner(dcel).iter().map(|points| {
        Polygon::new(LineString::from(Vec::from_iter(points.iter().map(|point| {
            to_point(*point)
        }))), vec!())
    }))
}

pub fn make_line_segments(dcel: &DCEL) -> Vec<Line<f64>> {
    Vec::from_iter(make_line_segments_inner(dcel).iter().map(|segment| {
        Line::new(to_point(segment[0]), to_point(segment[1]))
    }))
}

fn to_point(point: VoronoiPoint) -> geo_types::Point<f64> {
    Point::new(point.x.into(), point.y.into())
}
