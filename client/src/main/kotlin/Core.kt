import exception.DocumentError
import exception.GameException
import org.w3c.dom.Element
import kotlin.browser.document
import kotlin.browser.window

const val LOGIC_HZ: Double = 60.0 // logic tics per second
const val STEP_DELTA_MS: Double = 1000.0 / LOGIC_HZ // simulation ms per tic

const val DELTA_T_LIMIT_MS: Double = 1e4


/**
 * The game Core object contains the game main loop, and handles timing,
 * and calling of logic updates, and rendering.
 */
class Core {

    data class Tic(val timeStamp: Double, val timeStep: Double, val core: Core)

    companion object {
        val logger = Logger.getLogger("Core")
    }

    val container: Element = document.getElementById("container")
            ?: throw DocumentError("Could not find 'container' for game Core.")

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

    val input: InputHandler = InputHandler(container)
    val connection: ServerConnection = ServerConnection()
    val eventHandler: EventHandler = EventHandler()

    var deltaTMs: Double = 0.0 // ms simulation time that has not yet been run
    var lastFrameTimeMs: Double = 0.0 // timestamp of last loop start

    init {
        Logger.getLogger("Core").info("Initializing Core")

        // call setter (I hope there's a less weird way to do this)
        scene = scene
    }

    /**
     * Update function; called in a loop.
     * Calls logic tic updates, and render.
     */
    fun update(timeStamp: Double) {
        // As currently implemented, logic tics always comprise a fixed
        // time-span, and mean logic tic rate should be independent of
        // the render performance (physics will be processed at the same
        // rate regardless of time taken to render), however, due to the
        // variable number of logic tics per render, screen stutter may
        // become apparent with high-speed objects. If this becomes an
        // issue, interpolation should be implemented.

        // for reference:
        // https://isaacsukin.com/news/2015/01/detailed-explanation-javascript-game-loops-and-timing

        if (lastFrameTimeMs > 0) {
            deltaTMs += timeStamp - lastFrameTimeMs
        }
        lastFrameTimeMs = timeStamp

        if (deltaTMs > DELTA_T_LIMIT_MS) {
            throw GameException(
                    "Update DeltaT exceeded limit: $DELTA_T_LIMIT_MS ms.\n" +
                    "This is likely caused by a timing death spiral, in " +
                    "which simulation time passes faster than the time " +
                    "required to update logic"
            )
        }

        // run logic until it is caught up with current time
        while (deltaTMs >= STEP_DELTA_MS) {
            input.startTic() // update input buffers
            val tic = Tic(timeStamp - deltaTMs, STEP_DELTA_MS, this)
            scene.update(tic) // run logic tic
            deltaTMs -= STEP_DELTA_MS
        }

        scene.render()
        window.requestAnimationFrame({ t -> update(t) }) // request next frame
    }

    override fun toString() = "GameCore"
}


lateinit var core: Core


/**
 * Main function; called at startup
 */
fun main(args: Array<String>) {
    if (js("Module.ready") != true) {

        Logger.getLogger("Core").info("Module not yet ready.")
        window.setTimeout(
                { main(args) },
                500
        )
        return // come back later
    }
    try {
        core = Core()
    } catch (e: DocumentError) {
        return // don't execute, but don't quit
    }
    Logger.getLogger("Core").info("Began main loop")
    core.update(0.0)
}