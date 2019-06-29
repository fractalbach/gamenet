
use cgmath::Vector3;
use tectonic::TectonicLayer;


pub struct World {
    tectonic: TectonicLayer
}


impl World {
    pub fn new(seed: u32) -> World {
        World {
            tectonic: TectonicLayer::new(seed)
        }
    }

    pub fn height(&mut self, v: Vector3<f64>) -> f64 {
        assert_ne!(v, Vector3::new(0.0, 0.0, 0.0));

        self.tectonic.height(v)
    }
}
