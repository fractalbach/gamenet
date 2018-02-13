package objects.intf

import com.curiouscreature.kotlin.math.Double3

interface Movable : Positionable{
    var motion: Double3

    fun applyMotion() {
        position += motion
    }
}