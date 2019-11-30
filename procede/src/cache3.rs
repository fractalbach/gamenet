
use cgmath::{Vector2, Vector3, Vector4};


// --------------------------------------------------------------------


/// Cache for a spherical surface.
///
/// Provides samples by bilinear interpolation between cached points.
struct BilinearSphereCache {
    lru: LruCache<Vector4<i64>, Cube>,
}


impl BilinearSphereCache {

}
