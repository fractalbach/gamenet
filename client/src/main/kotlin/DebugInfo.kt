import org.w3c.dom.Element
import kotlin.browser.document

/**
 * Class handing display of fields in the webpage at runtime.
 *
 * Meant to help in debugging by displaying live information
 * about the state of objects in the engine.
 *
 * Will likely not be active in release.
 */
class DebugInfo {
    private val fields: MutableMap<String, String> = HashMap()
    private val element: Element? = document.getElementById("debugInfo")
    private var isActive = true

    init {
        val msg = "DebugElement " +
                (if (element != null) "Found" else "not found")
        Logger.getLogger("DebugInfo").info(msg)
    }

    /**
     * Set a debug field.
     *
     * @param k: Name of field.
     * @param v: String representation of variable.
     */
    operator fun set(k: String, v: String) {
        fields[k] = v
    }

    /**
     * Get displayed debug field.
     */
    operator fun get(k: String): String? {
        return fields[k]
    }

    fun removeField(k: String) {
        fields.remove(k)
    }

    fun disable() {
        element?.innerHTML = ""
        isActive = false
    }

    fun enable() {
        isActive = true
    }

    /**
     * Update displayed debug fields.
     */
    fun update() {
        if (element == null || !isActive) {
            return  // If debug container is missing,
        }

        var s = ""
        for (k in fields.keys.sorted()) {
            s += "$k: ${fields[k]}<br>"
        }

        element.innerHTML = s
    }
}
