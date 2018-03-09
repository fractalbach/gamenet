package objects

import info.laht.threekt.core.Object3D
import info.laht.threekt.lights.Light


/**
 * Class handling omnipresent ambient light.
 *
 * This light is cast onto all surfaces regardless of position, and
 * can be used as a rough approximation of scattered light.
 * To be convincing however, it must remain at some fraction of
 * light cast from more directional sources such as a SunLight,
 * point-light or other.
 */
class AmbientLight(name:String="", id:String="") : GameObject(name, id) {

    var light = js("new THREE.AmbientLight(0xFDB813, 0.5);") as Light
    override var threeObject: Object3D = light

    init {
        light.castShadow = false
    }
}