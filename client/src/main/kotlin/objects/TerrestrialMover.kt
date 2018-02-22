package objects

import Scene

import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.normalize
import info.laht.threekt.math.Vector3

/**
 * Superclass for all classes that are intended to be located on a
 * spheroid surface.
 * This class contains methods and fields that are used by subclasses.
 */
abstract class TerrestrialMover(name: String="", id: String=""):
        GameObject(name, id) {

    override var scene: Scene? = null
        set(scene) {
            super.scene = scene
            terrain = scene?.terrain
        }

    var terrain: Terrain? = null

    var sphereNormal: Double3 = Double3()
        get() {
            val terrain = terrain?:
                throw IllegalStateException("$this terrain property not set")
            return normalize(position - terrain.position)
        }

    /**
     * Rights the object, so that object Z axis points away from
     * center of spheroid.
     */
    protected open fun right() {
        val axis = Vector3(0.0, 0.0, 1.0)
        val normal = sphereNormal
        threeObject.quaternion.setFromUnitVectors(
                axis, Vector3(normal.x, normal.y, normal.z))
    }

    /**
     * Applies gravitational acceleration to object.
     */
    protected open fun applyGravity(tic: Core.Tic) {
        val scene = scene?:
            throw IllegalStateException("$this scene property not set")
        val deltaVelocities = sphereNormal * -scene.gravity * tic.timeStep / 1000.0
        motion += deltaVelocities
    }
}
