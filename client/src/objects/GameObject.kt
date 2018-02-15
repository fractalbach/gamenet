package objects

import Scene

/**
 * Abstract object from which other game types are extended.
 *
 * A GameObject is any object that is owned by a scene, and interact
 * with other parts of the scene. This includes objects that are not
 * visually represented, and/or have no position within the scene; for
 * example, logic controllers.
 */
abstract class GameObject(val name: String="", id: String="") {
    val id: String = if (id.isEmpty()) js("uuid()")!! else id
    var scene: Scene? = null

    /**
     * Method called each logical tic.
     */
    open fun update() {
        // empty here, extended by subclasses
    }

    override fun toString(): String {
        val className: String = this::class.simpleName!!
        val abbreviatedID: String = id.slice(id.length - 8 .. id.length)
        return "$className['$abbreviatedID]"
    }
}