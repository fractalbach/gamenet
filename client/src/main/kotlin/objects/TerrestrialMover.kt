package objects

import Scene

import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.normalize
import com.curiouscreature.kotlin.math.radians
import info.laht.threekt.math.Matrix3
import info.laht.threekt.math.Quaternion
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
            return normalize(worldPosition - terrain.worldPosition)
        }

    /**
     * Rights the object, so that object Z axis points away from
     * center of spheroid. (Sets Object to perfectly upright position)
     */
    protected open fun right() {
        val axis = Vector3(0.0, 0.0, 1.0)
        val v = Vector3()
        val normal = sphereNormal
        val normalVector = Vector3()
        normalVector.x = normal.x
        normalVector.y = normal.y
        normalVector.z = normal.z
        val worldQuat = Quaternion()
        threeObject.getWorldQuaternion(worldQuat)
        v.copy(axis).applyQuaternion(worldQuat)
        val adjustment = Quaternion()
        adjustment.setFromUnitVectors(v, normalVector)
        threeObject.applyQuaternion(adjustment)
    }

    /**
     * Sets object elevation above terrain surface to 0.0
     */
    protected open fun snapToSurface() {
        try {
            val norm_position:Double3 = normalize(sphereNormal)
            position = norm_position * (terrain!!.radius +
                    terrain!!.heightAtVector(norm_position))
        } catch (e: NullPointerException) {
            throw IllegalStateException("$this terrain property was not set")
        }
    }

    /**
     * Applies gravitational acceleration to object.
     */
    protected open fun applyGravity(tic: Core.Tic) {
        val scene = scene?:
            throw IllegalStateException("$this scene property not set")
        val deltaVelocities = sphereNormal * -scene.gravity *
                tic.timeStep / 1000.0
        motion += deltaVelocities
    }
}
