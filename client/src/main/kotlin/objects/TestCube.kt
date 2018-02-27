package objects

import info.laht.threekt.core.Object3D
import info.laht.threekt.geometries.BoxGeometry
import info.laht.threekt.materials.Material
import info.laht.threekt.materials.MeshStandardMaterial
import info.laht.threekt.math.Color
import info.laht.threekt.objects.Mesh

class TestCube(name: String="", id: String=""): GameObject(name, id) {
    override var threeObject: Object3D = makeMesh()

    init {
        threeObject.castShadow = true
        threeObject.receiveShadows = true
    }

    private fun makeMesh(): Mesh {
        val geometry = BoxGeometry(1, 1, 1, 1)
        val material = MeshStandardMaterial()
        material.color = Color(0x00ff00)
        // work around error in three.js wrapper; will be fixed soon
        @Suppress("CAST_NEVER_SUCCEEDS")
        val mesh = Mesh(geometry, material as Material)
        return mesh
    }
}