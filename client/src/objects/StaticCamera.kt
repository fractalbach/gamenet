package objects

import com.curiouscreature.kotlin.math.DMat3
import com.curiouscreature.kotlin.math.Double3
import objects.intf.Camera
import objects.intf.Orientable
import objects.intf.Positionable
import objects.intf.ThreeOwner

class StaticCamera(name: String="", id: String=""):
        GameObject(name, id),
        ThreeOwner,
        Camera,
        Positionable,
        Orientable
{
    override var position: Double3 = Double3()
    override var orientation: DMat3 = DMat3()

    override var threeObject: dynamic = null // todo: set tile obj

    // todo
}