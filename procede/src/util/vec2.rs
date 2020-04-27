use cgmath::{Vector2, vec2};
use geo_types::{CoordinateType, Point};
use num_traits::Float;


pub trait VecOps<T: CoordinateType + Float + Copy> {
    fn to_point(&self) -> Point<T>;

    fn rotate(&self, rad: T) -> Vector2<T>;
}

impl<T> VecOps<T> for Vector2<T>
    where T: CoordinateType + Float + Copy
{
    fn to_point(&self) -> Point<T> {
        Point::new(self.x, self.y)
    }

    fn rotate(&self, rad: T) -> Vector2<T> {
        let sin = rad.sin();
        let cos = rad.cos();
        vec2(
            (cos * self.x) - (sin * self.y),
            (sin * self.x) + (cos * self.y)
        )
    }
}
