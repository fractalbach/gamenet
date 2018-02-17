package objects

import Scene
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.length
import info.laht.threekt.core.Object3D

/**
 * Abstract object from which other game types are extended.
 *
 * A GameObject is any object that is owned by a scene, and interact
 * with other parts of the scene. This includes objects that are not
 * visually represented, and/or have no position within the scene; for
 * example, logic controllers.
 */
abstract class GameObject(val name: String="", id: String="") {
    val id: String = if (id.isEmpty()) js("uuid()") as String else id
    val childObjects = HashSet<GameObject>()
    var scene: Scene? = null
        set(scene) {
            if (scene == field) {
                return
            }
            field = scene
            scene?.threeScene?.add(threeObject)
            childObjects.forEach { child -> child.scene = scene }
        }

    abstract var threeObject: Object3D

    var position: Double3
        get() = Double3(threeObject.position)
        set(pos) {
            threeObject.position.x = pos.x
            threeObject.position.y = pos.y
            threeObject.position.z = pos.z
        }

    var visible: Boolean
        get() = threeObject.visible
        set(b) { threeObject.visible = b }
    
    fun translateX(value: Double) = threeObject.translateX(value)
    fun translateY(value: Double) = threeObject.translateY(value)
    fun translateZ(value: Double) = threeObject.translateZ(value)

    /**
     * Method called each logical tic.
     */
    open fun update(tic: Core.Tic) {
        // empty here, extended by subclasses
    }

    fun addChild(child: GameObject) = childObjects.add(child)


    fun distance(other: GameObject): Double {
        return length(other.position - position)
    }

    override fun toString(): String {
        val className: String = this::class.simpleName!!
        val abbreviatedID: String = id.slice(id.length - 8 .. id.length)
        return "$className['$abbreviatedID]"
    }
}