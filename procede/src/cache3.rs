
use cgmath::{Vector2, Vector3, Vector4};


// --------------------------------------------------------------------


struct GradientCache {
    lru: LruCache<Vector3<i64>, Cube>,
}

struct GradientCube {
    values: [Vector; 8]
}