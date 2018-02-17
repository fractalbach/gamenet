package objects

import info.laht.threekt.cameras.PerspectiveCamera
import info.laht.threekt.core.Object3D

const val DEFAULT_FAR_PLANE: Double = 1e4
const val DEFAULT_NEAR_PLANE: Double = 0.25


class Camera(name: String="", id: String=""): GameObject(name, id) {
    companion object {
        val logger = Logger.getLogger("Camera")
    }

    var threeCamera: info.laht.threekt.cameras.Camera =
            PerspectiveCamera(
                    60, 1 / 0.7, DEFAULT_NEAR_PLANE, DEFAULT_FAR_PLANE)
    override var threeObject: Object3D = threeCamera

    override fun update(tic: Core.Tic) {
        super.update(tic)
        if (tic.core.input.cmdActive(InputHandler.Command.MOVE_UP)) {
            val movement = -1.0 * tic.timeStep / 1000.0
            translateZ(movement)
        }
        if (tic.core.input.cmdActive(InputHandler.Command.MOVE_DOWN)) {
            val movement = 1.0 * tic.timeStep / 1000.0
            translateZ(movement)
        }
    }
}
