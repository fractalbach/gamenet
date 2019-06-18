
extern crate cgmath;
extern crate num_traits;

mod voronoi;
mod surface;
mod tectonic;

type Vec3 = cgmath::Vector3<f64>;

// --------------------------------------------------------------------
// Traits


trait HeightGen {
    fn height(&self, v: Vec3) -> f64;
}

trait NormalGen {
    fn normal(&self, v: Vec3) -> Vec3;
}

trait Layer {
    fn height(&self, h0: f64, v: Vec3) -> f64;
}


// --------------------------------------------------------------------
// Implementations


struct World {
    layers: [Box<Layer>]
}


struct BasicLayer {
    noise: noise::Perlin
}


// World --------------------------------------------------------------


// BasicLayer ---------------------------------------------------------
