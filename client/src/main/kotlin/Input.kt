import com.curiouscreature.kotlin.math.Double2
import org.w3c.dom.Element
import org.w3c.dom.events.Event
import org.w3c.dom.events.KeyboardEvent
import kotlin.browser.document
import kotlin.browser.window

private const val KEY_ARR_SIZE = 223


/**
 * Handles user key presses, and sets flags that can be checked
 * by various game objects when .update() is called.
 *
 * The purpose of this class is to allow asynchronous user input via
 * events to be read reliably, and quickly, by objects during the
 * game loop.
 */
class InputHandler(private val container: Element) {

    /**
     * Enum of keys that can be pressed, and their associated key codes.
     */
    enum class Key(val i: Int) {
        LEFT(37), UP(38), RIGHT(39), DOWN(40),
        A(65), B(66), C(67), D(68), E(69), F(70), G(71),
        H(72), I(73), J(74), K(75), L(76), M(77), N(78),
        O(79), P(80), Q(81), R(82), S(83), T(84), U(85),
        V(86), W(87), X(88), Y(89), Z(90),
        KEY0(48), KEY1(49), KEY2(50), KEY3(51), KEY4(52),
        KEY5(53), KEY6(54), KEY7(55), KEY8(56), KEY9(57),
        BACK(8), TAB(9), ENTER(13), SHIFT(16), CTRL(17), ALT(18),
        SPACE(32)

    }

    /**
     * Enum of actions that can be bound to a key.
     */
    enum class Command {
        MOVE_LEFT, MOVE_UP, MOVE_RIGHT, MOVE_DOWN;

        private val boundKeys: HashSet<Key> = HashSet()

        internal fun isPressed(input: InputHandler): Boolean {
            return boundKeys.any { input.keyPresses[it.i] }
        }

        internal fun isActive(input: InputHandler): Boolean {
            return boundKeys.any { input.keyStates[it.i] }
        }

        internal fun isReleased(input: InputHandler): Boolean {
            return boundKeys.any { input.keyReleases[it.i] }
        }

        internal fun bindKey(key: Key) = boundKeys.add(key)

        internal fun removeKey(key: Key) = boundKeys.remove(key)

        internal fun clearKeys() = boundKeys.clear()
    }

    private val keyStates: BooleanArray = BooleanArray(KEY_ARR_SIZE)

    private var keyPresses: BooleanArray = BooleanArray(KEY_ARR_SIZE)
    private var keyPressBuffer: BooleanArray = BooleanArray(KEY_ARR_SIZE)
    private var keyReleases: BooleanArray = BooleanArray(KEY_ARR_SIZE)
    private var keyReleaseBuffer: BooleanArray = BooleanArray(KEY_ARR_SIZE)

    private var pointerCaptured: Boolean = false

    var mouseMotion = Double2()
    private var mouseMotionBuffer = Double2()

    init {
        window.addEventListener(
                "keydown",
                { event: Event -> onKeyPressed(event as KeyboardEvent)}
        )
        window.addEventListener(
                "keyup",
                { event: Event -> onKeyReleased(event as KeyboardEvent)}
        )
        @Suppress("UNUSED_VARIABLE") // used in js
        val el = container
        @Suppress("UNUSED_VARIABLE") // used in js
        val onclickFunc = this::capturePointer
        js("el.onclick = onclickFunc")
        document.addEventListener(
                "pointerlockchange", this::onPointerLockChange, false)
        document.addEventListener(
                "mozpointerlockchange", this::onPointerLockChange, false)
        document.addEventListener(
                "webkitpointerlockchange", this::onPointerLockChange, false)

        // make default key mappings.
        // if customizable command maps are implemented later, this
        // should be moved into a separate class
        bindKey(Key.W, Command.MOVE_UP)
        bindKey(Key.A, Command.MOVE_LEFT)
        bindKey(Key.S, Command.MOVE_DOWN)
        bindKey(Key.D, Command.MOVE_RIGHT)
    }

    private fun onKeyPressed(event: KeyboardEvent) {
        val keyCode = event.keyCode
        keyPressBuffer[keyCode] = true
        keyStates[keyCode] = true
    }

    private fun onKeyReleased(event: KeyboardEvent) {
        val keyCode = event.keyCode
        keyReleaseBuffer[keyCode] = true
        keyStates[keyCode] = false
    }

    /** Returns whether key is has been pressed since the last logic tic */
    fun keyPressed(key: Key): Boolean = keyPresses[key.i]

    /** Returns whether the key is currently active / pressed down */
    fun keyActive(key: Key): Boolean = keyStates[key.i]

    /** Returns whether the key has been released since the last logic tic */
    fun keyReleased(key: Key): Boolean = keyReleases[key.i]

    /** Tests whether a command has been activated since the last logic tic */
    fun cmdPressed(cmd: Command): Boolean = cmd.isPressed(this)

    /** Returns whether a command is currently active / pressed down */
    fun cmdActive(cmd: Command): Boolean = cmd.isActive(this)

    /** Tests whether a command has been deactivated since the last logic tic */
    fun cmdReleased(cmd: Command): Boolean = cmd.isReleased(this)

    /** Binds passed key to passed command. */
    fun bindKey(key: Key, cmd: Command) = cmd.bindKey(key)

    /** Removes binding of passed key to passed command. */
    fun removeKey(key: Key, cmd: Command) = cmd.removeKey(key)

    /** Removes all key bindings. */
    fun clearKeys() = Command.values().forEach { cmd -> cmd.clearKeys() }


    private fun capturePointer() {
        @Suppress("UNUSED_VARIABLE") // used in js
        val el = container // bring element into local js scope
        js(
                "el.requestPointerLock = " +
                "el.requestPointerLock || " +
                "el.mozRequestPointerLock || " +
                "el.webkitRequestPointerLock;")
        js("el.requestPointerLock()")
    }

    @Suppress("UNUSED_PARAMETER")
    private fun onPointerLockChange(e: dynamic) {
        @Suppress("UNUSED_VARIABLE") // used in js
        val el = container // bring element into local js scope
        if (js("document.pointerLockElement === el ||" +
                "document.mozPointerLockElement === el ||" +
                "document.webkitPointerLockElement === el") as Boolean) {
            pointerCaptured = true
            document.addEventListener(
                    "mousemove", this::onMouseMove, false)
        } else {
            pointerCaptured = false
            document.removeEventListener(
                    "mousemove", this::onMouseMove, false)
        }
    }

    @Suppress("UNUSED_PARAMETER") // used in js
    private fun onMouseMove(e: dynamic) {
        if (!pointerCaptured) {
            return
        }
        mouseMotionBuffer = Double2(
            js("e.movementX || e.mozMovementX || e.webkitMovementX || 0.0")
                    as Double,
            js("e.movementY || e.mozMovementY || e.webkitMovementY || 0.0")
                    as Double
        )
    }

    /**
     * Called at start of logic tic, input received after this is
     * saved for the next logic tic, in order to ensure homogeneous
     * input given to all objects that need to check input, and to
     * ensure no input is cleared before being read.
     */
    fun startTic() {
        val oldKeyPresses = keyPresses
        val oldKeyReleases = keyReleases
        keyPresses = keyPressBuffer // switch buffers
        keyReleases = keyReleaseBuffer // switch buffers
        for (i in 0 until oldKeyPresses.size) {
            oldKeyPresses[i] = false // reset old array
            oldKeyReleases[i] = false // reset old array
        }
        // keyPressBuffer must be able to be assigned to at any time.
        keyPressBuffer = oldKeyPresses
        keyReleaseBuffer = oldKeyReleases

        // swap mouse buffer
        val oldMouseMotion = mouseMotion
        oldMouseMotion.x = 0.0
        oldMouseMotion.y = 0.0
        mouseMotion = mouseMotionBuffer
        mouseMotionBuffer = oldMouseMotion
    }
}
