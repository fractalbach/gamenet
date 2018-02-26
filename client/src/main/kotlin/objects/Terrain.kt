package objects

import Logger.Companion.getLogger
import com.curiouscreature.kotlin.math.Double2
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.normalize
import exception.CException
import info.laht.threekt.THREE.BackSide
import info.laht.threekt.core.Object3D
import info.laht.threekt.geometries.PlaneGeometry
import info.laht.threekt.materials.Material
import info.laht.threekt.materials.MeshStandardMaterial
import info.laht.threekt.math.Color
import info.laht.threekt.math.Vector3
import info.laht.threekt.objects.Mesh

private const val TERRAIN_SEED: Int = 124
private const val RADIUS: Double = 6.371e6
private const val HEIGHT_SCALE: Double = 1e4
private const val MAX_LOD: Int = 24 // any value up to 28
private const val MAX_ENCODED_LOD: Int = 28 // max LOD able to be encoded

// distance in tile widths at which a tile subdivides
private const val REL_SUBDIVISION_DIST: Double = 3 * RADIUS // must be > tile w
private const val TILE_POLYGON_WIDTH: Int = 8 // width in polygons of tile
private const val TILE_ROW_SIZE = TILE_POLYGON_WIDTH + 1
private const val N_TILE_VERTICES: Int = TILE_ROW_SIZE * TILE_ROW_SIZE

private const val MAX_TILE_DIVISIONS_PER_TIC = 32


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

    var subdivisionCounter = MAX_TILE_DIVISIONS_PER_TIC

    init {
        // add each face to scene
        faces.forEach {face -> addChild(face) }

        // initialize terrain module
        val echo: Int = js("_ter_TestEcho(4)") as Int
        if (echo != 4) {
            throw CException("Test Function call to C failed. " +
                    "Is Module set up?")
        }

        js("_ter_Init($TERRAIN_SEED, $RADIUS, $HEIGHT_SCALE)")
    }

    override fun update(tic: Core.Tic) {
        subdivisionCounter = MAX_TILE_DIVISIONS_PER_TIC
    }

    fun get(index: Int): Tile = faces[index]

    fun heightAtVector(vector: Double3): Double {
        return js("_ter_GetHeight(" +
                "vector.x, vector.y, vector.z, $MAX_LOD)") as Double *
                HEIGHT_SCALE
    }
}


class Tile(val terrain: Terrain, val face: Int,
           val parent: Tile?=null, val quadrant: Int?=null):
        GameObject()
{
    companion object {
        val logger = Logger.getLogger("Tile")

        /**
         * Gets tile-relative position from tile vertex index
         */
        fun tilePosFromVertIndex(i: Int): Double2 {
            return Double2(
                    i % TILE_ROW_SIZE.toDouble() / TILE_POLYGON_WIDTH,
                    (i / TILE_ROW_SIZE).toDouble() / TILE_POLYGON_WIDTH
            )
        }
    }
    val lod: Int = if (parent == null) 1 else parent.lod + 1
    val shape: Double2 = if (parent != null) parent.shape / 2.0 else
        Double2(2.0, 2.0)
    val relativeWidth = shape.x / 2  // 1.0 is diameter of spheroid
    val subTiles: Array<Tile?> = arrayOfNulls<Tile?>(4)
    val subdivisionDistance = REL_SUBDIVISION_DIST * relativeWidth
    val recombinationDistance = REL_SUBDIVISION_DIST * relativeWidth * 1.2

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

    override var threeObject: Object3D = makeThreeTile()

    init {
        logger.fine("created tile, face: $face, quad: $quadrant")
        logger.fine("position: $position")
        terrain.threeObject.add(threeObject) // add tile as child of terrain
        if (parent != null && quadrant == null) {
            throw IllegalArgumentException(
                    "If parent arg is passed, quadrant must also be passed.")
        }
    }

    /**
     * Updates Tile; if distance to camera is small enough, subdivides
     * tile to create more detail, or if already subdivided and camera
     * is far enough, recombines sub-tiles.
     */
    override fun update(tic: Core.Tic) {
        val dist = distance(scene!!.camera)
        if (dist < subdivisionDistance &&
                subTiles[0] == null &&
                lod < MAX_LOD &&
                terrain.subdivisionCounter > 0) {
            subdivide()
            terrain.subdivisionCounter -= 1
        } else if (dist > recombinationDistance && subTiles[0] != null) {
            recombine()
        }
    }

    /**
     * Finds lower left corner of tile, as a position relative to face.
     * Ex: lowest left position is (-1, -1) center is (0, 0).
     * Lower right corner is (1, -1).
     */
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
        visible = false // hide tile until a lower LOD is needed again
    }

    /**
     * Recombines tile, removing sub-tiles
     */
    private fun recombine() {
        for ((i, tile) in subTiles.withIndex()) {
            scene!!.remove(tile!!)
            subTiles[i] = null
        }
        visible = true
    }

    private fun makeThreeTile(): Mesh {

        /**
         * Creates geometry of tile.
         * Returns Pair of PlaneGeometry, and tile center position
         */
        fun makeGeometry(): Pair<PlaneGeometry, Double3> {
            try {
                // create position array.
                val geometry = PlaneGeometry(1, 1, 8, 8)
                val sphereRelativePositions: Array<Double3> = Array(
                        N_TILE_VERTICES, {
                    try {
                        val tileRelPos = tilePosFromVertIndex(it)
                        val facePos: Double2 = p1 + tileRelPos * shape
                        val cubeRelPos: Double3 = facePosTo3d(facePos)
                        val norm_pos: Double3 = normalize(cubeRelPos)
                        @Suppress("UNUSED_VARIABLE") // used in js
                        val x: Double = norm_pos.x;
                        @Suppress("UNUSED_VARIABLE") // used in js
                        val y: Double = norm_pos.y
                        @Suppress("UNUSED_VARIABLE") // used in js
                        val z: Double = norm_pos.z
                        val height: Double =
                            js("_ter_GetHeight(x, y, z, $MAX_LOD)") as Double
                        val pos = norm_pos * (RADIUS + height * HEIGHT_SCALE)
                        pos
                    } catch (e: Exception) {
                        logger.error("Error converting height index: $it")
                        throw e
                    }
                })

                val relativeCenter: Double3 =
                        sphereRelativePositions[N_TILE_VERTICES / 2]
                for (i in 0 until N_TILE_VERTICES) {
                    var pos = sphereRelativePositions[i]
                    pos -= relativeCenter
                    @Suppress("UNUSED_VARIABLE") // used in js
                    val v = Vector3(pos.x, pos.y, pos.z)
                    js("geometry.vertices[i] = v")
                }
                geometry.computeVertexNormals()
                return Pair(geometry, relativeCenter)
            } catch (e: Exception) {
                logger.error("Error creating $this geometry")
                throw e
            }
        }

        fun makeMaterial(): Material {
            val planeMaterial = MeshStandardMaterial()
            planeMaterial.color = Color(0x3cff00)
            // work around temporary error in THREE.js wrapper
            @Suppress("CAST_NEVER_SUCCEEDS")
            (planeMaterial as Material).side = BackSide
            //planeMaterial.wireframe = true // for debugging
            return planeMaterial
        }

        val (geometry: PlaneGeometry, tilePosition: Double3) = makeGeometry()
        val material: Material = makeMaterial()
        val mesh = Mesh(geometry, material)
        mesh.position.x = tilePosition.x
        mesh.position.y = tilePosition.y
        mesh.position.z = tilePosition.z
        mesh.updateMatrix()
        return mesh
    }

    private fun facePosTo3d(facePos: Double2): Double3 {
        return when (face) {
            0 -> Double3(1.0, facePos.x, facePos.y)
            1 -> Double3(-facePos.x, 1.0, facePos.y)
            2 -> Double3(-1.0, -facePos.x, facePos.y)
            3 -> Double3(facePos.x, -1.0, facePos.y)
            4 -> Double3(-facePos.y, facePos.x, 1.0)
            5 -> Double3(facePos.y, facePos.x, -1.0)
            else -> throw IllegalStateException("Face: $face")
        }
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
        for (i in 1 until lod) {
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
