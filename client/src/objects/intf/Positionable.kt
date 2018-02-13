package objects.intf

import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.length


/**
 * Component handling position information (not orientation)
 */
interface Positionable {
    var position: Double3

    fun distance(other: Positionable): Double {
        return length(other.position - position)
    }
}