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
use itertools::Itertools;
use std::iter::FromIterator;

use cgmath::{Vector2, vec2};
use geo_booleanop::boolean::BooleanOp;
use geo_types::{Polygon, Point, Line};
use geo::contains::Contains;
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
    nucleus: Point<f64>,
    bounds: Polygon<f64>,
}

pub struct LotPolyIterator<'a> {
    lot_poly: &'a LotPoly,
    i: i8,
    sub_iter: Option<Box<LotPolyIterator<'a>>>,
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
        LotPoly {
            poly,
            connections,
            width: settings.width,
            depth: settings.depth,
            lots,
            sub_poly: vec!(),
        }
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
        let lot_polygons = Self::create_lot_polygons(nuclei.clone(), poly);

        let mut lots = vec!();
        for (&nucleus, poly) in nuclei.iter().zip_eq(lot_polygons.iter()) {
            lots.push(Lot::new(nucleus, poly.clone()));
        }
        lots
    }

    fn create_lot_nuclei(
        poly: &Polygon<f64>, connections: &Vec<bool>, settings: &LotSettings
    ) -> Vec<Point<f64>> {
        debug_assert!(
            poly.exterior().num_coords() - 1 == connections.len(),
            "Connections vec length must match the number of polygon \
            exterior edges. Got {} edges and connections len {}.",
            poly.exterior().num_coords() - 1,
            connections.len()
        );

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
            let nuclei_offset = right * settings.depth / 2.;  // from face.

            // Split bounding polygon edge into lot faces.
            let edge_len = edge.length();
            let n_lot_faces = (edge_len / settings.width) as i32;
            if n_lot_faces < 1 {
                continue;
            }
            let faces = edge.divide(n_lot_faces);

            // Add lot nucleus for each lot face if there is room.
            for face in faces.lines() {
                let midpoint = face.midpoint();
                let lot_center = midpoint.to_vec() + nuclei_offset;

                // Check nucleus is not too close to another.
                if map.nearest(lot_center, settings.width).is_some() {
                    continue;
                }

                // Check nucleus is within polygon.
                // (Weird poly shapes are possible.)
                if !poly.contains(&lot_center.to_point()) {
                    continue;
                }

                // Check lot has enough space behind the nucleus.
                if !poly.contains(
                    &(midpoint.to_vec() + nuclei_offset * 2.).to_point()
                ) {
                    continue;
                }

                // Add nucleus.
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
        debug_assert!(nuclei.iter().all(|nucleus|{
            bounding_poly.contains(nucleus)
        }));
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
    pub fn divide(&mut self, settings: &LotSettings) {
        self.lots.clear();
        let ((poly_a, poly_b), (i0, i1)) = self.poly.halve();
        let mut a_conn = Vec::with_capacity(poly_a.exterior().num_coords());
        let mut b_conn = Vec::with_capacity(poly_b.exterior().num_coords());
        for &conn_state in self.connections.iter().skip(i1) {
            a_conn.push(conn_state);
        }
        for &conn_state in self.connections.iter().take(i0) {
            a_conn.push(conn_state);
        }
        a_conn.push(true);
        for &conn_state in self.connections.iter().skip(i0).take(i1 - i0) {
            b_conn.push(conn_state);
        }
        b_conn.push(true);

        // Produce sub-poly.
        self.sub_poly = vec![
            LotPoly::new(poly_a, a_conn, settings),
            LotPoly::new(poly_b, b_conn, settings),
        ]
    }

    pub fn n_lots(&self) -> usize {
        if !self.sub_poly.is_empty() {
            debug_assert!(self.lots.is_empty());
            self.sub_poly[0].n_lots() + self.sub_poly[1].n_lots()
        } else {
            self.lots.len()
        }
    }

    pub fn is_divided(&self) -> bool {
        !self.sub_poly.is_empty()
    }

    pub fn recurse(&self) -> LotPolyIterator {
        LotPolyIterator::new(self)
    }
}

impl Lot {
    fn new(nucleus: Point<f64>, poly: Polygon<f64>) -> Lot {
        Lot {
            nucleus,
            bounds: poly
        }
    }
}

impl<'a> LotPolyIterator<'a> {
    fn new(lot_poly: &'a LotPoly) -> LotPolyIterator<'a> {
        LotPolyIterator {
            lot_poly,
            i: 0,
            sub_iter: None
        }
    }
}

impl<'a> Iterator for LotPolyIterator<'a> {
    type Item = &'a LotPoly;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == 0 {
            self.i += 1;
            return Some(self.lot_poly)
        }
        while self.i < 3 {
            if self.sub_iter.is_none() {
                if self.lot_poly.is_divided() {
                    self.sub_iter = Some(Box::new(LotPolyIterator::new(
                        &self.lot_poly.sub_poly[(self.i - 1) as usize]
                    )));
                } else {
                    self.i = 3;
                    return None;
                }
            }
            let next = self.sub_iter.as_mut().unwrap().next();
            match next {
                Some(lot_poly) => {
                    return Some(lot_poly)
                },
                None => {
                    self.i += 1;
                    self.sub_iter = None;
                }
            }
        }
        None  // i == 3.
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
            (x: -104., y: 95.),
            (x: 204., y: 101.),
            (x: 200., y: -40.),
            (x: -100., y: -10.),
        ];
        let connections = vec!(true, true, true, true);
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let poly = LotPoly::new(poly, connections, &settings);
        assert_gt!(poly.lots.len(), 4);

        serialize_to(&poly, "lot_division.json");
    }

    #[test]
    fn test_lot_division_with_concave_side() {
        let poly = polygon![
            (x: -104., y: 95.),
            (x: 204., y: 101.),
            (x: 200., y: -40.),
            (x: 20., y: 35.),
            (x: -100., y: -10.),
        ];
        let connections = vec!(true, true, true, true, true);
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let poly = LotPoly::new(poly, connections, &settings);
        assert_gt!(poly.lots.len(), 4);

        serialize_to(&poly, "lot_division_concave.json");
    }

    #[test]
    fn test_lot_division_with_rounded_poly() {
        let poly = polygon![
            (x: -104., y: 95.),
            (x: 89., y: 154.),
            (x: 204., y: 101.),
            (x: 200., y: -40.),
            (x: 20., y: -75.),
            (x: -100., y: -10.),
        ];
        let connections = vec!(true, true, true, true, true, true);
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let poly = LotPoly::new(poly, connections, &settings);
        assert_gt!(poly.lots.len(), 4);

        serialize_to(&poly, "lot_division_rounded.json");
    }

    #[test]
    fn test_lot_division_with_unconnected_edge() {
        let poly = polygon![
            (x: -104., y: 155.),
            (x: 204., y: 55.),
            (x: 200., y: -40.),
            (x: -100., y: -10.),
        ];
        let connections = vec!(true, true, true, false);
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let poly = LotPoly::new(poly, connections, &settings);
        assert_gt!(poly.lots.len(), 4);

        serialize_to(&poly, "lot_division_unconnected_edge.json");
    }

    #[test]
    fn test_polygon_division() {
        let poly = polygon![
            (x: -104., y: -10.),
            (x: -80., y: 30.),
            (x: -40., y: 50.),
            (x: -00., y: 60.),
            (x: 30., y: 50.),
            (x: 60., y: 10.),
            (x: 70., y: -10.),
            (x: 50., y: -5.),
            (x: 20., y: 0.),
            (x: -10., y: 0.),
            (x: -35., y: 0.),
            (x: -70., y: 0.),
        ];
        let connections = vec!(
            false, false, false, false, false, false,
            true, true, true, true, true, true,
        );
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let mut poly = LotPoly::new(poly, connections, &settings);
        let n0 = poly.n_lots();
        poly.divide(&settings);
        let n1 = poly.n_lots();
        assert_gt!(n1, n0);

        serialize_to(&poly, "lot_polygon_division.json");
    }

    #[test]
    fn test_multiple_polygon_division() {
        let poly = polygon![
            (x: -312., y: -20.),
            (x: -240., y: 90.),
            (x: -120., y: 150.),
            (x: 0., y: 180.),
            (x: 90., y: 150.),
            (x: 180., y: 30.),
            (x: 210., y: -30.),
            (x: 150., y: -15.),
            (x: 60., y: 0.),
            (x: -30., y: 0.),
            (x: -105., y: 0.),
            (x: -210., y: 0.),
        ];
        let connections = vec!(
            false, false, false, false, false, false,
            true, true, true, true, true, true,
        );
        let settings = LotSettings {
            width: 16.,
            depth: 20.,
            cost_mod_fn: &|a: Vector2<f64>, b: Vector2<f64>| 1.,
        };
        let mut poly = LotPoly::new(poly, connections, &settings);
        let n0 = poly.n_lots();
        poly.divide(&settings);
        let n1 = poly.n_lots();
        assert_gt!(n1, n0);

        serialize_to(&poly, "lot_polygon_division2.json");
    }
}
