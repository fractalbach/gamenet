package objects

import Core
import InputHandler
import Scene
import info.laht.threekt.core.Object3D
import info.laht.threekt.geometries.BoxGeometry
import info.laht.threekt.materials.Material
import info.laht.threekt.materials.MeshStandardMaterial
import info.laht.threekt.math.Color
import info.laht.threekt.objects.Mesh

private const val MOUSE_TRAVERSE_COEF = 0.003


class TestMover(name: String="", id: String=""): TerrestrialMover(name, id) {
    override var threeObject: Object3D = makeMesh()


    init {
        threeObject.castShadow = true
        threeObject.receiveShadows = true
    }

    override var scene: Scene?
        get() = super.scene
        set(scene) {
            super.scene = scene
            snapToSurface()
            right()
        }

    private fun makeMesh(): Mesh {
        val geometry = BoxGeometry(1, 1, 1, 1)
        val material = MeshStandardMaterial()
        material.color = Color(0x00ff00)
        // work around error in three.js wrapper; will be fixed soon
        @Suppress("CAST_NEVER_SUCCEEDS")
        val mesh = Mesh(geometry, material as Material)
        return mesh
    }

    override fun update(tic: Core.Tic) {
        super.update(tic)
        var moved: Boolean = false
        if (tic.core.input.cmdActive(InputHandler.Command.MOVE_UP)) {
            val movement = 8.0 * tic.timeStep / 1000.0
            translateY(movement)
            moved = true
        }
        if (tic.core.input.cmdActive(InputHandler.Command.MOVE_DOWN)) {
            val movement = -8.0 * tic.timeStep / 1000.0
            translateY(movement)
            moved = true
        }
        if (moved) {
            snapToSurface()
            right()
        }

        val traverse: Double = tic.core.input.mouseMotion.x
        if (traverse != 0.0) {
            val euler = rotation
            euler.z -= MOUSE_TRAVERSE_COEF * tic.core.input.mouseMotion.x
            rotation = euler
        }
    }
}