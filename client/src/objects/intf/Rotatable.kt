package objects.intf

import com.curiouscreature.kotlin.math.DMat3

interface Rotatable : Orientable {
    var rotation: DMat3

    fun applyRotation() {
        orientation *= rotation
    }
}