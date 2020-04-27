use cgmath::{Vector2, vec2};
use geo_types::{CoordinateType, Point};
use quad::{Spatial, Rect};


pub trait PointOps<T: CoordinateType> {
    fn to_vec(&self) -> Vector2<T>;
}


impl<T> PointOps<T> for Point<T>
where T: CoordinateType
{
    fn to_vec(&self) -> Vector2<T> {
        vec2(self.x(), self.y())
    }
}
