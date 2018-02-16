package objects

import Logger.Companion.getLogger
import com.curiouscreature.kotlin.math.DMat3
import com.curiouscreature.kotlin.math.Double2
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.normalize
import exception.CException
import info.laht.threekt.THREE.DoubleSide
import info.laht.threekt.core.Object3D
import info.laht.threekt.geometries.PlaneGeometry
import info.laht.threekt.materials.Material
import info.laht.threekt.materials.MeshBasicMaterial
import info.laht.threekt.materials.MeshStandardMaterial
import info.laht.threekt.math.Color
import info.laht.threekt.math.Vector3
import info.laht.threekt.objects.Mesh
import org.khronos.webgl.Float64Array
import org.khronos.webgl.get

private const val RADIUS: Double = 6.371 //e6
private const val MAX_LOD: Int = 20 // any value up to 28
private const val MAX_ENCODED_LOD: Int = 28 // max LOD able to be encoded

// distance in tile widths at which a tile subdivides
private const val REL_SUBDIVISION_DIST: Double = 3.0
private const val TILE_POLYGON_WIDTH: Int = 8 // width in polygons of tile
private const val N_TILE_VERTICES: Int =
    (TILE_POLYGON_WIDTH + 1) * (TILE_POLYGON_WIDTH + 1)


/**
 * Terrain
 */
open class Terrain(id: String=""): GameObject("Terrain", id) {
    companion object {
        val logger = getLogger("Terrain")
    }

    override var threeObject: Object3D = Object3D() // nothing special

    val radius = RADIUS
    val faces: Array<Tile> = Array(6, {i -> Tile(this, i)})

    init {
        // add each face to scene
        faces.forEach {face -> addChild(face) }
    }

    fun get(index: Int): Tile = faces[index]
}


class Tile(val terrain: Terrain, val face: Int,
           val parent: Tile?=null, val quadrant: Int?=null):
        GameObject()
{
    companion object {
        val logger = Logger.getLogger("Tile")
    }
    val lod: Int = if (parent == null) 1 else parent.lod + 1
    val shape: Double2 = if (parent != null) parent.shape / 2.0 else
        Double2(2.0, 2.0)
    val subTiles: Array<Tile?> = arrayOfNulls<Tile?>(4)

    override var threeObject: Object3D = makeThreeTile() // todo: set tile obj

    /*
     * Array's first value is the index of the tile's face,
     * and each following integer is the quadrant index of each sub-tile
     * containing the Tile, until the last index, which indicates the
     * quadrant of the Tile.
     */
    val quadrants: Array<Int> = Array(
            lod,
            {i ->
                when {
                    i < lod - 1 -> parent!!.quadrants[i]
                    i == 0 -> face
                    else -> quadrant!!
                }
            }
    )

    val p1 = findP1() // lower left corner, relative to cube face
    val p2 = p1 + shape // upper right corner, relative to cube face

    init {
        if (parent != null && quadrant == null) {
            throw IllegalArgumentException(
                    "If parent arg is passed, quadrant must also be passed.")
        }
    }

    override fun update() {
        val dist = distance(scene!!.camera)
        if (dist / REL_SUBDIVISION_DIST < 1 &&
                subTiles[0] == null &&
                lod < MAX_LOD) {
            subdivide()
        } else if (dist / REL_SUBDIVISION_DIST > 1 && subTiles[0] != null) {
            recombine()
        }
    }

    private fun findP1(): Double2 {
        if (parent == null) {
            return Double2(-1.0, -1.0)
        }
        return when (quadrant!!) {
            0 -> parent.p1 + shape // middlepoint
            1 -> Double2(parent.p1.x, parent.p1.y + shape.y)
            2 -> parent.p1
            3 -> Double2(parent.p1.x + shape.x, parent.p1.y)
            else ->throw IllegalArgumentException()
        }
    }

    /**
     * Subdivides tile into quadrants with higher LOD
     */
    private fun subdivide() {
        var tile: Tile
        for (i in subTiles.indices) {
            tile = Tile(terrain, face=face, parent=this, quadrant=i)
            scene!!.add(tile)
            subTiles[i] = tile
        }
        // visible = false // hide tile until a lower LOD is needed again
    }

    /**
     * Recombines tile, removing sub-tiles
     */
    private fun recombine() {
        for ((i, tile) in subTiles.withIndex()) {
            scene!!.remove(tile!!)
            subTiles[i] = null
        }
        // visible = true
    }

    private fun makeThreeTile(): Mesh {

        fun makeGeometry(): PlaneGeometry {
            try {
                //val positions = Float64Array(N_TILE_VERTICES)
                //val result = js("_ter_HeightFromPosCode(ptr, posCode)")
                //if (!result) {
                //    throw CException("error calling ter_HeightFromPosCode()")
                //}
                // once height array has been received, create
                // position array.
                val geometry = PlaneGeometry(1, 1, 8, 8)
                val vertWidth = TILE_POLYGON_WIDTH + 1
                for (i in 0 until N_TILE_VERTICES) {
                    try {
                        val height = 0.0 //positions[i]
                        val tileRelPos = Double2(
                                i % vertWidth.toDouble() / vertWidth,
                                (i / vertWidth).toDouble() / vertWidth
                        )
                        val facePos: Double2 = tileRelPos * shape
                        val cubeRelPos: Double3 = when (face) {
                            0 -> Double3(1.0, facePos.x, facePos.y)
                            1 -> Double3(-facePos.x, 1.0, facePos.y)
                            2 -> Double3(-1.0, -facePos.x, facePos.y)
                            3 -> Double3(facePos.x, -1.0, facePos.y)
                            4 -> Double3(-facePos.y, facePos.x, 1.0)
                            5 -> Double3(facePos.y, facePos.x, -1.0)
                            else -> throw IllegalStateException("Face: $face")
                        }
                        val pos: Double3 = normalize(cubeRelPos) *
                                (terrain.radius + height)
                        @Suppress("UNUSED_PARAMETER")
                        val v = Vector3(pos.x, pos.y, pos.z)
                        js("geometry.vertices[i] = v")
                    } catch (e: Exception) {
                        logger.error("Error converting height index: $i")
                        throw e
                    }
                }
                return geometry
            } catch (e: Exception) {
                logger.error("Error creating $this geometry")
                throw e
            }
        }

        fun makeMaterial(): Material {
            val planeMaterial = MeshStandardMaterial()
            planeMaterial.color = Color(0xffff00)
            // work around temporary error in THREE.js wrapper
            @Suppress("CAST_NEVER_SUCCEEDS")
            (planeMaterial as Material).side = DoubleSide
            // todo: modify
            return planeMaterial
        }

        val geometry: PlaneGeometry = makeGeometry()
        val material: Material = makeMaterial()
        val threeTile = Mesh(geometry, material)

        return threeTile
    }

    // getters + setters

    /**
     * Creates a long value unique to this tile's position, that
     * can be deconstructed to determine the position of a tile.
     *
     * This is intended to be used both as a unique identifier for a
     * tile, to be used when caching data, as well as a means of
     * communicating position of the tile to C++.
     */
    fun getPositionCode(): Long {
        // 5b: lod, 3b: face, 2b * LOD: quadrants
        if (lod > MAX_ENCODED_LOD) { //  28 unless encoding method changes
            throw IllegalStateException("Too many LOD to encode: $lod")
        }
        val faceBits: Long = face.toLong() shl 5
        var positionCode: Long = faceBits or lod.toLong()
        for (i in 0 until lod) {
            val shift = 8 + 2 * i
            val quadrant = quadrants[i]
            positionCode = (quadrant.toLong() shl shift) or positionCode
        }
        return positionCode
    }
}

/**
 * Reconstructs tile quadrants array from passed tile position code.
 */
fun getPositionFromCode(encodedPos: Long): Pair<Int, Array<Int>> {
    // 5b: lod, 3b: face, 2b * LOD: quadrants
    val nQuadrants: Int = (encodedPos and 0x1F).toInt()
    val face: Int = ((encodedPos shr 5) and 0x7).toInt()
    val quadrants: Array<Int> = Array(
            nQuadrants, {i -> ((encodedPos shr 8 + 2 * i) and 0x3).toInt()})
    return Pair(face, quadrants)
}


// TODO: Scene.update should be able to add / remove game objects while
// iterating over objects.
