use cgmath::{Vector2, vec2};
use geo_types::{CoordinateType, Point, Coordinate};
use num_traits::Float;


pub trait VecOps<T: CoordinateType + Float + Copy> {
    fn to_point(&self) -> Point<T>;

    fn rotate(&self, rad: T) -> Vector2<T>;
}

pub trait ToVec2<T: CoordinateType + Float + Copy> {
    fn to_vec(&self) -> Vector2<T>;
}

impl<T> VecOps<T> for Vector2<T>
    where T: CoordinateType + Float + Copy
{
    fn to_point(&self) -> Point<T> {
        Point::new(self.x, self.y)
    }

    fn rotate(&self, rad: T) -> Vector2<T> {
        let rad = -rad;
        let sin = rad.sin();
        let cos = rad.cos();
        vec2(
            (cos * self.x) - (sin * self.y),
            (sin * self.x) + (cos * self.y)
        )
    }
}

impl<T> ToVec2<T> for Point<T>
    where T: CoordinateType + Float + Copy
{
    fn to_vec(&self) -> Vector2<T> {
        vec2(self.x(), self.y())
    }
}

impl<T> ToVec2<T> for Coordinate<T>
    where T: CoordinateType + Float + Copy
{
    fn to_vec(&self) -> Vector2<T> {
        vec2(self.x, self.y)
    }
}


#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use cgmath::{Vector2, vec2};

    use ::util::vec2::VecOps;

    #[test]
    fn test_basic_rotate() {
        let rad = PI / 3.;
        let result = vec2(0., 1.).rotate(rad);
        assert_vec2_near!(result, vec2(rad.sin(), rad.cos()));
    }
}
