package objects

import info.laht.threekt.core.Object3D
import info.laht.threekt.lights.Light

private const val DEFAULT_SHADOW_MAP_SIZE = 256


/**
 * Class handling mono-directional light.
 */
class SunLight(name:String="", id:String="") : GameObject(name, id) {

    var light = js("new THREE.DirectionalLight(0xFDB813, 0.7);") as Light
    override var threeObject: Object3D = light

    init {
        light.castShadow = true
//        val l = light  // bring light into local scope for javascript
//        val shadowMapSize = DEFAULT_SHADOW_MAP_SIZE
//        js("l.shadow.mapSize.width = l.shadow.mapSize.height = shadowMapSize")
    }
}