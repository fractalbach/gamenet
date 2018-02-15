import exception.DocumentError
import org.w3c.dom.Element
import kotlin.browser.document

class Core {
    val container: Element? = document.getElementById("container")

    var scene: Scene = Scene("Main Scene", this)
        // setter for scene switches out scene object, adjusts document,
        // and gives Scene instance a reference to the game core.
        set(scene) {
            container ?: throw DocumentError(
                    "Could not find 'container' for game Core.")
            scene.core = this
            container.innerHTML = "" // remove placeholder text
            // todo: remove previous scene if it is present
            container.appendChild(scene.renderer.domElement!!)
            field = scene
        }

    fun update() {
        scene.update()
    }
}


val core: Core = Core()


/**
 * Main function; called at startup
 */
fun main(args: Array<String>) {
    val message = "Hello Kotlin"
    println(message)
}


/**
 * Update function; called in a loop.
 */
fun update() {
    core.update()
}
