package objects

import Core
import Scene
import Logger
import Logger.Companion.getLogger
import com.curiouscreature.kotlin.math.Double2
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.normalize
import exception.CException
import info.laht.threekt.THREE.BackSide
import info.laht.threekt.core.Object3D
import info.laht.threekt.geometries.PlaneBufferGeometry
import info.laht.threekt.materials.Material
import info.laht.threekt.materials.MeshStandardMaterial
import info.laht.threekt.math.Color
import info.laht.threekt.objects.Mesh
import util.ObjectPool

private const val TERRAIN_SEED: Int = 124
private const val RADIUS: Double = 6.371e6
private const val HEIGHT_SCALE: Double = 1e4
private const val MAX_LOD: Int = 24 // any value up to 28
private const val MAX_ENCODED_LOD: Int = 28 // max LOD able to be encoded

// distance in tile widths at which a tile subdivides
private const val REL_SUBDIVISION_DIST: Double = 2 * RADIUS // must be > tile w
private const val TILE_POLYGON_WIDTH: Int = 8 // width in polygons of tile
private const val TILE_HEIGHT_ROW_SIZE = TILE_POLYGON_WIDTH + 1
private const val TILE_VERTICES_ROW_SIZE = TILE_HEIGHT_ROW_SIZE + 2
private const val N_TILE_HEIGHTS: Int =
        TILE_HEIGHT_ROW_SIZE * TILE_HEIGHT_ROW_SIZE
private const val N_TILE_VERTICES: Int =
        TILE_VERTICES_ROW_SIZE * TILE_VERTICES_ROW_SIZE

private const val TILE_LIP_BASE_SCALE: Double = 1.0 / TILE_POLYGON_WIDTH
private const val MAX_TILE_DIVISIONS_PER_TIC = 32


/**
 * Terrain GameObject that manages and acts as parent to terrain tiles.
 *
 * Terrain will maintain a hierarchy of terrain Tiles that provide
 * varying levels of detail depending on camera distance.
 */
open class Terrain(id: String=""): GameObject("Terrain", id) {
    companion object {
        private val logger = getLogger("Terrain")
    }

    override var threeObject: Object3D = Object3D() // nothing special

    val radius = RADIUS
    val faces: Array<Tile> = Array(6) { Tile(this, it) }
    /** Stores tiles no longer in use, but whose geometry can be reused */
    private val tilePile: ObjectPool<Tile> = ObjectPool {
        Tile(this, 0)
    }

    var subdivisionCounter = MAX_TILE_DIVISIONS_PER_TIC

    /** Gravitational acceleration of terrestrial objects */
    val gravity: Double = 9.806

    init {
        js("Module.terrain = this")
        // add each face to scene
        faces.forEach { addChild(it) }

        // initialize terrain module
        val echo: Int = js("_ter_TestEcho(4)") as Int
        if (echo != 4) {
            throw CException("Test Function call to C failed. " +
                    "Is Module set up?")
        }

        js("_ter_Init($TERRAIN_SEED, $RADIUS, $HEIGHT_SCALE)")
    }

    /**
     * Updates terrain, checking to see which tiles owned by Terrain
     * require subdivision, and which can be recombined.
     * @param tic: Core.Tic
     */
    override fun update(tic: Core.Tic) {
        subdivisionCounter = MAX_TILE_DIVISIONS_PER_TIC
    }

    /**
     * Cleans up at end of update tic.
     *
     * Performs needed upkeep.
     */
    override fun updateEnd(tic: Core.Tic) {
        tilePile.upkeep()
    }

    /**
     * Gets tile identified by passed index.
     *
     * 0 - 3 are tiles describing the equator of the spheroid, with 0
     * index being the Tile centered on the x+ axis direction, and 1,
     * 2, and 3 proceeding counter-clockwise, when viewed from the top
     * (tile 1 being at y+, and so on.) Tile 4 is the z+ tile, and
     * Tile 5 is the z- tile.
     *
     * @param index: Int index of tile which serves as the
     *              associated face.
     */
    fun get(index: Int): Tile = faces[index]

    /**
     * Adds passed tile to discard bin, from which tiles can be re-used
     * instead of letting them be garbage collected.
     */
    fun addTileToBin(tile: Tile) {
        tile.active = false
        tilePile.recycle(tile)
    }

    /**
     * Gets a Tile for use.
     * Will return a previously created, but unused tile if one is
     * available, otherwise a new Tile is created.
     */
    fun getTile(face: Int, parent: Tile? = null, quadrant: Int? = null): Tile {
        val tile: Tile = tilePile.get()
        tile.active = true
        tile.setGeometry(face, parent, quadrant)
        return tile
    }

    /**
     * Gets height at surface position
     */
    @Suppress("UNUSED_PARAMETER") // used in js
    fun heightAtVector(vector: Double3): Double {
        return js("_ter_GetHeight(" +
                "vector.x, vector.y, vector.z, $MAX_LOD)") as Double *
                HEIGHT_SCALE
    }
}


/**
 * Tiles are procedurally generated segments of Terrain surface that
 * subdivide and recombine to provide varying levels of detail
 * depending on distance to camera.
 */
class Tile(private val terrain: Terrain, face: Int,
           parent: Tile? = null, quadrant: Int? = null) :
        GameObject()
{

    companion object {
        private val logger = Logger.getLogger("Tile")

        /**
         * Gets tile-relative position from tile vertex index.
         * @param i: Int index of height value in height array.
         * @return Double2 with ranges from (0.0,0.0) to (1.0,1.0)
         */
        fun tilePosFromHeightIndex(i: Int): Double2 {
            if (i < 0 || i >= N_TILE_HEIGHTS) {
                throw IllegalArgumentException(
                        "index $i outside ${0 until N_TILE_HEIGHTS}")
            }
            return Double2(
                    i % TILE_HEIGHT_ROW_SIZE.toDouble() / TILE_POLYGON_WIDTH,
                    (i / TILE_HEIGHT_ROW_SIZE).toDouble() / TILE_POLYGON_WIDTH
            )
        }

        /**
         * Returns x and y position of passed vertex index in a tile,
         * followed by boolean indicating whether index is on a tile lip.
         *
         * Helper function for makeGeometry().
         * @param i: Index of vertex in vertex array.
         * @return Int index of height value in height array, and
         *              Boolean indicating whether or not passed
         *              vertex index is on a Tile lip.
         */
        fun vertexData(i: Int): Pair<Int, Boolean> {
            if (i < 0 || i >= N_TILE_VERTICES) {
                throw IllegalArgumentException(
                        "index $i outside ${0 until N_TILE_VERTICES}")
            }
            var x: Int = i % TILE_VERTICES_ROW_SIZE
            var y: Int = i / TILE_VERTICES_ROW_SIZE
            var isLip = false
            if (x == 0) {
                x++
                isLip = true
            } else if (x == TILE_VERTICES_ROW_SIZE - 1) {
                x--
                isLip = true
            }
            if (y == 0) {
                y++
                isLip = true
            } else if (y == TILE_VERTICES_ROW_SIZE - 1) {
                y--
                isLip = true
            }
            x--
            y--
            return Pair(y * TILE_HEIGHT_ROW_SIZE + x, isLip)
        }
    }

    var parent: Tile? = null
    var face: Int = -1
    var quadrant: Int? = null
    var active: Boolean = true

    /**
     * Tile level of detail, with Terrain Face being 1, the first face
     * subdivision being 2, etc.
     */
    private var lod: Int = -1
    /**
     * Relative shape of tile as compared to face. (1.0,1.0 indicates
     * that the Tile is the same size as the cube face, (0.5,0.5)
     * indicates it is half that, etc.
     */
    var shape: Double2 = Double2()
    private var relativeWidth: Double = -1.0  // 1.0 is diameter of spheroid
    private val subTiles: Array<Tile?> = arrayOfNulls<Tile?>(4)
    private var subdivisionDistance: Double = -1.0
    private var recombinationDistance: Double = -1.0

    var geometry: PlaneBufferGeometry? = null

    /**
     * Array's first value is the index of the tile's face,
     * and each following integer is the quadrant index of each sub-tile
     * containing the Tile, until the last index, which indicates the
     * quadrant of the Tile.
     */
    private lateinit var quadrants: Array<Int>

    /** lower left corner, relative to cube face */
    var p1 = Double2()
    /** upper right corner, relative to cube face */
    var p2 = Double2()

    /** THREE.js Plane mesh object */
    override var threeObject: Object3D = makeThreeTile()

    init {
        setGeometry(face, parent, quadrant)
        terrain.threeObject.add(threeObject) // add tile as child of terrain
        if (parent != null && quadrant == null) {
            throw IllegalArgumentException(
                    "If parent arg is passed, quadrant must also be passed.")
        }
    }

    /**
     * Creates Geometry given passed information.
     * @param face: Int index of face on which Tile resides
     * @param parent: Tile?
     * @param quadrant: Int
     */
    fun setGeometry(face: Int, parent: Tile? = null, quadrant: Int? = null) {
        try {
            logger.fine(
                    "creating tile geometry, face: $face, quad: $quadrant")
            if (parent === this) {
                throw IllegalArgumentException("Passed parent $parent == this.")
            }

            this.face = face
            this.parent = parent
            this.quadrant = quadrant

            lod = if (parent == null) 1 else parent.lod + 1

            quadrants = Array(lod) { i: Int ->
                when {
                    i < lod - 1 -> parent!!.quadrants[i]
                    i == 0 -> face
                    else -> quadrant!!
                }
            }

            shape = if (parent != null) parent.shape / 2.0 else
                Double2(2.0, 2.0)
            relativeWidth = shape.x / 2  // 1.0 is diameter of spheroid
            subdivisionDistance = REL_SUBDIVISION_DIST * relativeWidth
            recombinationDistance =
                    REL_SUBDIVISION_DIST * relativeWidth * 1.2
            p1 = findP1(parent, shape)
            p2 = p1 + shape

            val geometry: dynamic = this.geometry
                    ?: throw IllegalStateException("Geometry is null")
            geometry.verticesNeedUpdate = true
            geometry.attributes.position.needsUpdate = true

            val pos: Double3 = setVertices()
            geometry.computeBoundingSphere()

            threeObject.position.set(pos.x, pos.y, pos.z)
            threeObject.updateMatrix()
        } catch (e: Exception) {
            logger.error("Error occurred in Tile.setGeometry() method", e)
            throw e
        }
    }

    /**
     * Sets geometry positions
     */
    private fun setVertices(): Double3 {
        try {
            val spherePositions: Array<Double3> = Array(N_TILE_HEIGHTS) {
                try {
                    val tileRelPos = tilePosFromHeightIndex(it)
                    val facePos: Double2 = p1 + tileRelPos * shape
                    val cubeRelPos: Double3 = facePosTo3d(facePos)
                    val normPos: Double3 = normalize(cubeRelPos)
                    @Suppress("UNUSED_VARIABLE") // used in js
                    val x: Double = normPos.x
                    @Suppress("UNUSED_VARIABLE") // used in js
                    val y: Double = normPos.y
                    @Suppress("UNUSED_VARIABLE") // used in js
                    val z: Double = normPos.z
                    val height: Double =
                            js("_ter_GetHeight(x, y, z, $MAX_LOD)") as Double
                    val pos = normPos * (RADIUS + height * HEIGHT_SCALE)
                    pos
                } catch (e: Exception) {
                    logger.error("Error converting height index: $it")
                    throw e
                }
            }

            val vertPositions: Array<Double3> = Array(N_TILE_VERTICES) {
                val (heightIndex: Int, isLip: Boolean) = vertexData(it)
                // sanity check
                if (heightIndex < 0 || heightIndex >= N_TILE_HEIGHTS) {
                    throw IllegalStateException(
                            "bad height index: $heightIndex. vert: $it")
                }
                val heightRatio: Double = if (isLip)
                    1.0 - shape.x * TILE_LIP_BASE_SCALE else 1.0
                val vertexPosition: Double3 =
                        spherePositions[heightIndex] * heightRatio
                vertexPosition
            }

            val relativeCenter: Double3 = vertPositions[N_TILE_VERTICES / 2]
            @Suppress("UNUSED_VARIABLE") // used in js
            val geometry = this.geometry
            @Suppress("UNUSED_VARIABLE") // used in js
            val positionsArray =
                    js("geometry.getAttribute(\"position\")").array
            for (i in 0 until N_TILE_VERTICES) {
                var pos = vertPositions[i]
                pos -= relativeCenter
                @Suppress("UNUSED_VARIABLE") // used in js
                val vertexStartIndex: Int = i * 3
                js("positionsArray[vertexStartIndex] = pos.x")
                js("positionsArray[vertexStartIndex + 1] = pos.y")
                js("positionsArray[vertexStartIndex + 2] = pos.z")
            }
            return relativeCenter
        } catch (e: Exception) {
            logger.error("Error setting $this vertices")
            throw e
        }
    }

    /**
     * Updates Tile; if distance to camera is small enough, subdivides
     * tile to create more detail, or if already subdivided and camera
     * is far enough, recombines sub-tiles.
     * @param tic: Core.Tic
     */
    override fun update(tic: Core.Tic) {
        if (!active) {
            return
        }
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
     * @return Double2
     */
    private fun findP1(parent: Tile?, shape: Double2): Double2 {
        if (parent == null) {
            return Double2(-1.0, -1.0)
        }
        val quadrant: Int = this.quadrant ?: throw NullPointerException(
                "Tile.findP1() called when Tile quadrant is null")
        return when (quadrant) {
            0 -> parent.p1 + shape // middlepoint
            1 -> Double2(parent.p1.x, parent.p1.y + shape.y)
            2 -> parent.p1
            3 -> Double2(parent.p1.x + shape.x, parent.p1.y)
            else -> throw IllegalArgumentException()
        }
    }

    /**
     * Subdivides tile into quadrants with higher LOD
     */
    private fun subdivide() {
        try {
            var tile: Tile
            val scene: Scene = this.scene ?: throw IllegalStateException(
                    "Tile.subdivide(): No scene set.")
            for (i in subTiles.indices) {
                tile = terrain.getTile(face, this, i)
                if (tile == this) {
                    throw IllegalStateException("Identity Crisis.")
                }
                tile.visible = true
                if (!scene.contains(tile)) {
                    scene.add(tile)
                }
                subTiles[i] = tile
            }
            visible = false // hide tile until a lower LOD is needed again
        } catch (e: Exception) {
            logger.error("Error occurred while subdividing $this", e)
            throw e
        }
    }

    /**
     * Recombines tile, removing sub-tiles
     */
    private fun recombine() {
        // this may be called when no sub-tiles exist, if the parent
        // tile is being recombined.
        try {
            for ((i, tile) in subTiles.withIndex()) {
                if (tile != null) {
                    tile.recombine()
                    tile.visible = false
                    tile.position = Double3()
                    terrain.addTileToBin(tile)
                }
                subTiles[i] = null
            }
            visible = true
        } catch (e: Exception) {
            logger.error("Error occurred while recombining $this", e)
            throw e
        }
    }

    /**
     * Creates THREE.js Mesh for this Tile
     * @return Mesh
     */
    private fun makeThreeTile(): Mesh {

        /**
         * Creates geometry of tile.
         * Returns Pair of PlaneGeometry, and tile center position
         */
        fun makeGeometry(): PlaneBufferGeometry {
            // create position array.
            return PlaneBufferGeometry(1, 1, 10, 10)
        }

        fun makeMaterial(): Material {
            val planeMaterial = MeshStandardMaterial()
            planeMaterial.color = Color(0x3cff00)
            planeMaterial.side = BackSide
            //planeMaterial.wireframe = true // for debugging
            planeMaterial.flatShading = true
            return planeMaterial
        }

        val geometry: PlaneBufferGeometry = makeGeometry()
        this.geometry = geometry
        val material: Material = makeMaterial()
        val mesh = Mesh(geometry, material)
        mesh.matrixAutoUpdate = false // tile won't be moving very often
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
     * @return Long (int64) encoded position code.
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
            nQuadrants, { i -> ((encodedPos shr 8 + 2 * i) and 0x3).toInt() })
    return Pair(face, quadrants)
}
