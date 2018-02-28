package objects

import Core
import Scene
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.length
import info.laht.threekt.core.Object3D
import info.laht.threekt.math.Euler

/**
 * Abstract object from which other game types are extended.
 *
 * A GameObject is any object that is owned by a scene, and interacts
 * with other parts of the scene. This includes objects that are not
 * visually represented, and/or have no position within the scene; for
 * example, logic controllers.
 *
 * @see Scene
 */
abstract class GameObject(val name: String="", id: String="") {
    /**
     * String UUID that is unique to this GameObject in the client,
     * but shared across clients with other objects displaying the same
     * entity.
     */
    val id: String = if (id.isEmpty()) js("uuid()") as String else id
    /** GameObjects owned by this GameObject */
    val childObjects = HashSet<GameObject>()
    /** Scene in which GameObject resides */
    open var scene: Scene? = null
        set(scene) {
            if (scene == field) {
                return
            }
            field = scene
            scene?.threeScene?.add(threeObject)
            childObjects.forEach { child -> child.scene = scene }
        }

    /** THREE Object3d which is wrapped by GameObject */
    abstract var threeObject: Object3D

    /** Relative position of object in meters. Relative to parent
     * position if GameObject has a parent, otherwise
     * world position. */
    var position: Double3
        get() = Double3(threeObject.position)
        set(pos) {
            threeObject.position.x = pos.x
            threeObject.position.y = pos.y
            threeObject.position.z = pos.z
        }

    /** Position relative to world origin */
    var worldPosition: Double3
        get() = Double3(threeObject.getWorldPosition())
        set(v) = throw NotImplementedError()

    /** Motion of object in m/s */
    var motion: Double3 = Double3()

    /** Euler rotation of object relative to parent or world if object
     * has no parent. */
    var rotation: Euler
        get() = threeObject.rotation
        set(rot) = threeObject.setRotationFromEuler(rot)

    /** Indicates whether or not object is visible in game world */
    var visible: Boolean
        get() = threeObject.visible
        set(b) { threeObject.visible = b }

    /** Adjusts GameObject X position by passed value */
    fun translateX(value: Double) = threeObject.translateX(value)
    /** Adjusts GameObject Y position by passed value */
    fun translateY(value: Double) = threeObject.translateY(value)
    /** Adjusts GameObject Z position by passed value */
    fun translateZ(value: Double) = threeObject.translateZ(value)

    /**
     * Method called each logical tic.
     * @see Scene.update
     * @see Core.update
     * @param tic: Core.Tic containing timing information and
     *              other data.
     */
    open fun update(tic: Core.Tic) {
        // empty here, extended by subclasses
    }

    /**
     * Adds a child GameObject to the instance.
     * @param child: GameObject
     */
    fun addChild(child: GameObject) = childObjects.add(child)


    /**
     * Returns distance between worldPositions of this GameObject and
     * the passed GameObject.
     * @see GameObject.worldPosition
     * @param other: GameObject
     * @return Double distance in meters
     */
    fun distance(other: GameObject): Double {
        return length(other.worldPosition - worldPosition)
    }

    /** @suppress String representation of GameObject */
    override fun toString(): String {
        val className: String = this::class.simpleName!!
        val abbreviatedID: String = id.slice(id.length - 8 .. id.length)
        return "$className['$abbreviatedID]"
    }
}
