package lib.math

import com.curiouscreature.kotlin.math.Double3
import info.laht.threekt.math.Vector3

class Vec3(x: Double, y: Double, z: Double): Vector3(x, y, z) {
    constructor(): this(0.0, 0.0, 0.0)
    constructor(double3: Double3): this(double3.x, double3.y, double3.z)
}