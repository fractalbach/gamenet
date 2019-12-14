//! Map used to generate a tensor field which determines
//! street direction.

use aabb_quadtree::{QuadTree, ItemId};
use aabb_quadtree::geom::{Rect, Point, Vector};
use aabb_quadtree::Spatial;
use cgmath::{Vector2, vec2};

use pop::streets::util::{vec_to_point};


pub struct TensorField {
    map: QuadTree<InfluenceSource>
}

#[derive(Debug, Clone, Copy)]
pub struct InfluenceSource {
    form: InfluenceForm,
    bounds: Rect,
}

#[derive(Debug, Clone, Copy)]
pub enum InfluenceForm { Point, Line }


impl TensorField {
    pub fn new(bounds: Rect) -> TensorField {
        TensorField {
            map: QuadTree::default(bounds),
        }
    }
}


impl InfluenceSource {
    pub fn new(form: InfluenceForm, bounds: Rect) -> InfluenceSource {
        InfluenceSource { form, bounds }
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