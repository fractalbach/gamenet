/// Module handling division of a polygon into lots.
///
/// Multiple steps are encompassed here.
///
/// From an initial polygon and lot info (width + min depth), divides
/// polygon into cells, starting at polygon edges marked as connections.
///
/// Once cells are established, paths are made along cell edges to
/// reach poly edges marked as connections.
///
/// Space used by paths are subtracted from cells, yielding the final
/// lot sizes.
use std::iter::FromIterator;

use cgmath::{Vector2, vec2};
use geo_booleanop::boolean::BooleanOp;
use geo_types::{Polygon, Point, Line};
use serde::{Deserialize, Serialize};

use delaunay;
use quad::{QuadMap, Rect, Spatial};
use util::poly::PolyOps;
use util::line::LineOps;
use util::point::{PointOps, VecOps};
use util::vec::VecMap;


/// Settings used in lot generation.
///
/// # Fields
/// * width - Mean width of lots.
/// * depth - Mean depth of lots.
/// * tgt_lots - Number of lots to produce. If needed, the LotPoly will
///             subdivide to produce additional facing area. There is
///             no guarantee that the tgt number of lots will be
///             reached. The produced LotPoly should be examined if the
///             final number is important.
/// * division_fn - Function used to determine if the polygon or sub-
///             polygon should divide.
/// * cost_mod_fn - Function used to find cost mod for connecting
///             two points. Should be greater than 1.0 or negative.
///             Negative value indicates connection is impossible.
#[derive(Clone)]
pub struct LotSettings<'a> {
    pub width: f64,
    pub depth: f64,
    pub division_fn: &'a Fn(&LotPoly) -> bool,
    pub cost_mod_fn: &'a Fn(Vector2<f64>, Vector2<f64>) -> f64,
    // As const settings are required, they should be added here.
}


/// Struct handling production of lots within a polygonal space.
#[derive(Serialize, Deserialize)]
pub struct LotPoly {
    poly: Polygon<f64>,
    connections: Vec<bool>,
    width: f64,
    depth: f64,
    lots: Vec<Lot>,
    sub_poly: Vec<LotPoly>,
}


/// Struct with data about a single lot.
#[derive(Serialize, Deserialize)]
pub struct Lot {
    bounds: Polygon<f64>,
}


// --------------------------------------------------------------------


impl LotPoly {
    /// Produces a new LotPoly with passed polygon shape, connections,
    /// and settings.
    ///
    /// # Arguments
    /// * poly - Polygon to divide into lots. Must be wound clockwise.
    /// * connections - Vec of bool indicating whether each edge in
    ///             polygon is a connection.
    /// * settings - Const settings for producing LotPoly.
    ///
    /// # Return
    /// LotPoly
    pub fn new(
        poly: Polygon<f64>, connections: Vec<bool>, settings: &LotSettings
    ) -> LotPoly {
        // Produce initial lot distribution.
        let lots = Self::create_lots(&poly, &connections, settings);
        let mut lot_poly = LotPoly {
            poly,
            connections,
            width: settings.width,
            depth: settings.depth,
            lots,
            sub_poly: vec!(),
        };

        // Subdivide polygon if appropriate.
        if (settings.division_fn)(&lot_poly) {
            lot_poly.divide(settings);
        }

        lot_poly
    }

    /// Create lots for a given polygon and other data.
    ///
    /// This function does not split the polygon, it only divides an
    /// existing shape into lots.
    ///
    /// Assumes clockwise winding of polygon.
    fn create_lots(
        poly: &Polygon<f64>, connections: &Vec<bool>, settings: &LotSettings
    ) -> Vec<Lot> {
        let nuclei: Vec<Point<f64>> = Self::create_lot_nuclei(
            poly, connections, settings
        );
        let lot_polygons = Self::create_lot_polygons(nuclei, poly);

        lot_polygons.map(|poly| Lot::new(poly.clone()))
    }

    fn create_lot_nuclei(
        poly: &Polygon<f64>, connections: &Vec<bool>, settings: &LotSettings
    ) -> Vec<Point<f64>> {
        // Produce quad-map to store lot centers in.
        let bounds = poly.aabb();
        let mut map: QuadMap<Vector2<f64>> = QuadMap::default(bounds);

        // Place lot nuclei along each connected edge.
        let mut nuclei = vec!();
        for i in 0..(poly.exterior().num_coords() - 1) {
            if !connections[i] {
                continue;
            }
            let edge = Line::new(
                poly.exterior()[i],
                poly.exterior()[i + 1],
            );
            let right = edge.right();  // Right-side normal vec.
            let edge_len = edge.length();
            let n_lot_faces = (settings.width / edge_len) as i32;
            if n_lot_faces < 1 {
                continue;
            }
            let faces = edge.divide(n_lot_faces);
            for face in faces.lines() {
                let midpoint = face.midpoint();
                let lot_center = midpoint.to_vec() + right * settings.depth;
                if map.nearest(lot_center, settings.width).is_some() {
                    continue;
                }
                map.insert(lot_center);
                nuclei.push(lot_center.to_point());
            }
        }
        nuclei
    }

    fn create_lot_polygons(
            nuclei: Vec<Point<f64>>, bounding_poly: &Polygon<f64>
    ) -> Vec<Polygon<f64>> {
        if nuclei.is_empty() {
            return vec!()
        }
        let delaunay = delaunay::Delaunay::from_points(nuclei, 50.);
        let unclipped_polygons = delaunay.voronoi_polygons();
        let mut lot_polygons = vec!();
        for poly in unclipped_polygons {
            let intersections = poly.intersection(bounding_poly);
            match intersections.into_iter().next() {
                Some(intersection) => lot_polygons.push(intersection),
                None => ()
            }
        }
        lot_polygons
    }

    /// Divides a lot polygon into two.
    fn divide(&mut self, settings: &LotSettings) {
        // TODO
    }
}

impl Lot {
    fn new(poly: Polygon<f64>) -> Lot {
        Lot {
            bounds: poly
        }
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use std::f64;

    use pop::streets::lot::{LotPoly, LotSettings};
    use cgmath::Vector2;
    use test_util::serialize_to;

    #[test]
    fn test_simple_lot_division() {
        let poly = polygon![
            (x: -100., y: -10.),
            (x: 200., y: -40.),
            (x: 204., y: 101.),
            (x: -104., y: 95.),
        ];
        let connections = vec!(true, true, true, true);
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            division_fn: &|poly: &LotPoly| false,  // Don't divide.
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let poly = LotPoly::new(poly, connections, &settings);

        serialize_to(&poly, "lot_division.json");
    }
}
