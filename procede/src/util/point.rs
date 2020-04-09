use cgmath::{Vector2, vec2};
use geo_types::{CoordinateType, Point};


pub trait PointOps<T: CoordinateType> {
    fn to_vec(&self) -> Vector2<T>;
}

pub trait VecOps<T: CoordinateType> {
    fn to_point(&self) -> Point<T>;
}


impl<T> PointOps<T> for Point<T>
where T: CoordinateType
{
    fn to_vec(&self) -> Vector2<T> {
        vec2(self.x(), self.y())
    }
}

impl<T> VecOps<T> for Vector2<T>
where T: CoordinateType
{
    fn to_point(&self) -> Point<T> {
        Point::new(self.x, self.y)
    }
}
