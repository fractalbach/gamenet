use cgmath::Vector3;


pub trait Layer {
    type Result;

    fn sample(&self, v: Vector3<f64>) -> Self::Result;
}


pub trait Height {
    fn height(&self) -> f64;
}
