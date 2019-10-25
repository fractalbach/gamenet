
use cgmath::{Vector3, vec3};
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
        assert_ne!(v, vec3(0.0, 0.0, 0.0));

        let tectonic_info = self.tectonic.height(v);

        tectonic_info.height
    }
}
