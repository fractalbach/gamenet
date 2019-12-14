//! Map used to generate a tensor field which determines
//! street direction.

use aabb_quadtree::{QuadTree, ItemId};
use aabb_quadtree::geom::{Rect, Point, Vector};
use aabb_quadtree::Spatial;
use cgmath::{Vector2, vec2};

use pop::streets::util::{find_line_bounds, vec_to_point};


/// TensorField specialized for determining road direction
///
/// Uses InfluenceSource instances to determine points or lines of
/// interest around which road networks form.
pub struct TensorField {
    map: QuadTree<InfluenceSource>,
    globals: Vec<InfluenceSource>,
}

#[derive(Debug, Clone, Copy)]
pub struct InfluenceSource {
    form: InfluenceForm,
    bounds: Rect,
    v: f64
}

#[derive(Debug, Clone, Copy)]
pub enum InfluenceForm { Point, Line }


impl TensorField {
    const INFLUENCE_R: f32 = 1000.0;

    /// Create new TensorField
    ///
    /// # Arguments
    /// * `bounds` - Bounding box which contains all influences to
    ///             be added.
    /// # Return
    /// Created TensorField
    pub fn new(bounds: Rect) -> TensorField {
        TensorField {
            map: QuadTree::default(bounds),
            globals: Vec::with_capacity(1)
        }
    }

    /// Adds an InfluenceSource to the TensorField
    ///
    /// The added influence source will affect all samples taken within
    /// the field's influence radius (const).
    ///
    /// To make an influence global, add it using the
    /// add_global() method.
    ///
    /// # Arguments
    /// * `influence` - InfluenceSource to add
    pub fn add(&mut self, influence: InfluenceSource) {
        self.map.insert(influence);
    }

    /// Adds a global InfluenceSource to the TensorField
    ///
    /// The added influence source will affect all samples taken from
    /// the field. This is suitable for large influences such as city
    /// centers or other landmarks.
    ///
    /// To avoid needless computation, add objects with small influence
    /// effects using the add() method.
    ///
    /// # Arguments
    /// * `influence` - InfluenceSource to add
    pub fn add_global(&mut self, influence: InfluenceSource) {
        self.globals.push(influence);
    }

    /// Gets influence direction at passed position.
    ///
    /// The returned influence vector points away from sources
    /// of influence.
    ///
    /// # Arguments
    /// * `uv` - Position at which to sample the TensorField.
    ///
    /// # Return
    /// Influence vector pointing away from sources of influence.
    pub fn sample(&self, uv: Vector2<f64>) -> Vector2<f64> {
        let mut sum = vec2(0.0, 0.0);
        for global in &self.globals {
            sum += global.influence(uv);
        }
        let p = vec_to_point(uv);
        let query_rect = Rect::centered_with_radius(&p, Self::INFLUENCE_R);
        for (influence, _, _) in self.map.query(query_rect) {
            sum += influence.influence(uv);
        }

        sum
    }

    /// Convenience static function which returns the perpendicular
    /// Vector of the passed Vector.
    ///
    /// # Arguments
    /// * `v` - Vector to rotate.
    ///
    /// # Return
    /// Rotated perpendicular vector. Always points directly to the right of the
    /// Vector passed.
    pub fn right(v: Vector2<f64>) -> Vector2<f64> {
        vec2(-v.y, v.x)
    }
}


impl InfluenceSource {
    pub fn new(form: InfluenceForm, bounds: Rect, v: f64) -> InfluenceSource {
        InfluenceSource { form, bounds, v }
    }

    pub fn from_point(p: Vector2<f64>, v: f64) -> InfluenceSource {
        InfluenceSource::new(
            InfluenceForm::Point,
            Rect::null_at(&vec_to_point(p)),
            v,
        )
    }

    pub fn from_line(
        a: Vector2<f64>, b: Vector2<f64>, v: f64
    ) -> InfluenceSource {
        InfluenceSource::new(
            InfluenceForm::Line,
            find_line_bounds(a, b),
            v,
        )
    }

    /// Retrieves influence at passed uv coordinate.
    ///
    /// The returned influence vector points directly away from
    /// the InfluenceSource.
    ///
    /// # Arguments
    /// * `uv` - Position vector relative to the same origin as the
    ///             InfluenceSource bounds
    ///
    /// # Return
    /// * Vector
    pub fn influence(&self, uv: Vector2<f64>) -> Vector2<f64> {
        let dir = match self.form {
            InfluenceForm::Point => self.point_direction(uv),
            InfluenceForm::Line => self.point_direction(uv),  // TODO
        };
        let scale = self.falloff(dir.magnitude());
        let result = dir.normalized().scale_e(scale, scale);
        vec2(result.x as f64, result.y as f64)
    }

    /// Gets direction from InfluenceSource to a point.
    fn point_direction(&self, uv: Vector2<f64>) -> Vector {
        debug_assert!(self.bounds.top_left() == self.bounds.bottom_right());
        vec_to_point(uv) - self.bounds.top_left()
    }

    /// Returns weight from passed distance.
    fn falloff(&self, d: f32) -> f32 {
        1.0f32 / d  // May need to be replaced.
    }
}

impl Spatial for InfluenceSource {
    fn aabb(&self) -> Rect {
        self.bounds
    }
}
