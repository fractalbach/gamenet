package objects

import com.curiouscreature.kotlin.math.Double3
import info.laht.threekt.extras.SceneUtils

open class FollowCamera(name: String="", id: String=""): Camera(name, id) {

    var followDistance: Double = 3.0
    var followed: GameObject? = null

    fun follow(followed: GameObject) {
        if (this.followed != null) {
            SceneUtils.detach(threeObject, this.followed!!.threeObject, scene!!.threeScene)
        }
        position = Double3(0.0, -followDistance, followDistance / 2)
        SceneUtils.attach(threeObject, followed.threeObject, scene!!.threeScene)
        this.followed = followed
        logger.debug("$this (pos: $position) now following " +
                "$followed (pos: ${followed.position})")
    }
}