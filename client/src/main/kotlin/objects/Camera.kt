package objects

import Logger
import getSettings
import info.laht.threekt.cameras.Camera
import info.laht.threekt.cameras.PerspectiveCamera
import info.laht.threekt.core.Object3D

const val DEFAULT_FAR_PLANE: Double = 2e5
const val DEFAULT_NEAR_PLANE: Double = 0.25


open class Camera(name: String="", id: String=""): GameObject(name, id) {
    companion object {
        val logger = Logger.getLogger("Camera")
    }

    var threeCamera: Camera

    init {
        val settings = getSettings()
        threeCamera = PerspectiveCamera(
                60,
                1 / 0.7,
                settings.nearPlane?: DEFAULT_NEAR_PLANE,
                settings.farPlane?: DEFAULT_FAR_PLANE
        )
    }

    override var threeObject: Object3D = threeCamera
}
