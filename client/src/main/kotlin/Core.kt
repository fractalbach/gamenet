import exception.DocumentError
import org.w3c.dom.Element
import kotlin.browser.document
import kotlin.browser.window


class Core {
    companion object {
        val logger = Logger.getLogger("Core")
    }
    val container: Element = document.getElementById("container")?:
        throw DocumentError("Could not find 'container' for game Core.")

    var scene: Scene = Scene("Main Scene", this)
        // setter for scene switches out scene object, adjusts document,
        // and gives Scene instance a reference to the game getCore.
        set(scene) {
            scene.core = this
            container.innerHTML = "" // remove placeholder text
            // todo: remove previous scene if it is present
            container.appendChild(scene.renderer.domElement)
            field = scene
            logger.info("Set new scene: $scene")
        }

    init {
        // call setter (I hope there's a less weird way to do this)
        scene = scene
    }

    fun update() {
        scene.update()
        scene.render()
    }

    override fun toString() = "GameCore"
}


lateinit var core: Core


/**
 * Main function; called at startup
 */
fun main(args: Array<String>) {
    core = Core()
    Logger.getLogger("Core").info("Began main loop")
    update()
}


/**
 * Update function; called in a loop.
 */
fun update() {
    core.update()
    window.requestAnimationFrame({_ -> update()})
}
