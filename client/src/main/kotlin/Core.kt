import exception.DocumentError
import exception.GameException
import org.w3c.dom.Element
import kotlin.browser.document
import kotlin.browser.window

/** logic tics per second */
const val LOGIC_HZ: Double = 60.0
/** ms simulated per tic */
const val STEP_DELTA_MS: Double = 1000.0 / LOGIC_HZ

/** Maximum DeltaT, above which some action to correct or exit should occur */
const val DELTA_T_LIMIT_MS: Double = 1e4


/**
 * The game Core object contains the game main loop, and handles timing,
 * and calling of logic updates, and rendering.
 */
class Core {

    /**
     * Data-Class whose instances store information about specific
     * game tics.
     */
    data class Tic(val timeStamp: Double, val timeStep: Double, val core: Core)

    companion object {
        private val logger = Logger.getLogger("Core")
    }

    /**
     * Document element in which game runs and is displayed
     *
     */
    val container: Element = document.getElementById("container")
            ?: throw DocumentError("Could not find 'container' for game Core.")

    /** Current primary game scene. */
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

    /**
     * InputHandler in use by game; handles user mouse and
     * key actions.
     */
    val input: InputHandler = InputHandler(container)
    /** Handles server connection, receipt, and sending of Messages */
    val connection: ServerConnection = ServerConnection()
    /** Handles parsing of received events */
    val eventHandler: EventHandler = EventHandler()

    private var deltaTMs: Double = 0.0 // ms not-yet-run simulation time
    private var lastFrameTimeMs: Double = 0.0 // timestamp of last update call

    init {
        logger.info("Initializing Core")

        // call setter (I hope there's a less weird way to do this)
        scene = scene
    }

    /**
     * Update function; called in a loop.
     * Calls logic tic updates, and render.
     *
     * @param timeStamp: Timestamp in millis passed by browser, or
     *      not passed at all if update() is being called for the
     *      first time.
     * @throws GameException if timing failure occurs due to
     *      timing-death-spiral or similar.
     *
     * @see Scene.update
     */
    fun update(timeStamp: Double=0.0) {
        // As currently implemented, logic tics always comprise a fixed
        // time-span, and mean logic tic rate should be independent of
        // the render performance (physics will be processed at the same
        // rate regardless of time taken to render), however, due to the
        // variable number of logic tics per render, screen stutter may
        // become apparent with high-speed objects. If this becomes an
        // issue, interpolation should be implemented.

        // for reference:
        // https://isaacsukin.com/news/2015/01/detailed-explanation-javascript-game-loops-and-timing

        // get deltaT between last timestamp and current timestamp
        if (lastFrameTimeMs > 0) {
            deltaTMs += timeStamp - lastFrameTimeMs
        }
        lastFrameTimeMs = timeStamp

        // if logic has somehow fallen far behind where it should be
        // (1+ seconds) so that the situation seems non-recoverable,
        // exit with an error message.
        if (deltaTMs > DELTA_T_LIMIT_MS) {
            throw GameException(
                    "Update DeltaT exceeded limit: $DELTA_T_LIMIT_MS ms.\n" +
                    "This is likely caused by a timing death spiral, in " +
                    "which simulation time is passing faster than the time " +
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
        window.requestAnimationFrame({ update(it) }) // request next frame
    }

    /** @suppress */
    override fun toString() = "GameCore"
}


/**
 * Main function; called on load.
 * Function will check to see that Module containing wasm is ready,
 * and if not, reschedule startCore for a slightly later time.
 *
 * This function appears in JavaScript without name-mangling
 */
@JsName("startCore")
fun startCore(args: Array<String>) {

    if (js("Module.ready") != true) {

        Logger.getLogger("Core").info("Module not yet ready.")
        window.setTimeout(
                { startCore(args) },
                500
        )
        return // come back later
    }
    val core = Core()
    Logger.getLogger("Core").info("Began main loop")
    core.update(0.0)
}
