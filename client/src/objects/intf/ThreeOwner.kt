package objects.intf


/**
 * Component handling ownership of a THREE.js model.
 */
interface ThreeOwner {
    var threeObject: dynamic

    /**
     * Sets visibility of THREE.js object
     */
    fun setVisibility(b: Boolean) {
        // todo
    }
}
