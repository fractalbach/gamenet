package objects

import info.laht.threekt.cameras.PerspectiveCamera
import info.laht.threekt.core.Object3D

const val DEFAULT_FAR_PLANE: Double = 1e4
const val DEFAULT_NEAR_PLANE: Double = 0.25


class Camera(name: String="", id: String=""): GameObject(name, id) {

    var threeCamera: info.laht.threekt.cameras.Camera =
            PerspectiveCamera(
                    60, 1 / 0.7, DEFAULT_NEAR_PLANE, DEFAULT_FAR_PLANE)
    override var threeObject: Object3D = threeCamera
}
