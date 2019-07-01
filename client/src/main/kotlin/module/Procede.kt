package module

import com.curiouscreature.kotlin.math.Double3

/**
 * Class wrapping calls to the procede wasm module.
 */
class Procede(val seed: Int) {
    companion object {
        private val procede: dynamic = js("window.procede")

        fun ready(): Boolean {
            return procede.ready as Boolean
        }
    }

    private val worldId: Int = procede.exports.create_world(seed)

    fun height(v: Double3): Double {
        return procede.exports.height(worldId, v.x, v.y, v.z) as Double
    }

    fun release() {
        procede.exports.release_world(worldId)
    }
}
