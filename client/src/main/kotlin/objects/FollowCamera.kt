package objects

import Core
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.radians
import info.laht.threekt.math.Euler

private const val MOUSE_ELEVATION_COEF = 0.0015


open class FollowCamera(name: String="", id: String=""): Camera(name, id) {

    var followDistance: Double = 3.0
    var followed: GameObject? = null

    fun follow(followed: GameObject) {
        followed.threeObject.add(threeObject)
        this.followed = followed
        position = Double3(0.0, -followDistance, followDistance / 2)
        rotation = Euler(radians(60.0), 0.0, 0.0)
        logger.debug("$this (pos: $position) now following " +
                "$followed (pos: ${followed.position})")
    }

    override fun update(tic: Core.Tic) {
        super.update(tic)

        val traverse: Double = tic.core.input.mouseMotion.x
        if (traverse != 0.0) {
            val euler = rotation
            euler.x -= MOUSE_ELEVATION_COEF * tic.core.input.mouseMotion.y
            rotation = euler
        }
    }
}