package objects

import Core
import Scene
import Logger
import Logger.Companion.getLogger
import com.curiouscreature.kotlin.math.Double2
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.abs
import com.curiouscreature.kotlin.math.cross
import com.curiouscreature.kotlin.math.length
import com.curiouscreature.kotlin.math.normalize
import exception.CException
import getTerrainMat
import info.laht.threekt.THREE.BackSide
import info.laht.threekt.core.Object3D
import info.laht.threekt.geometries.PlaneBufferGeometry
import info.laht.threekt.materials.Material
import info.laht.threekt.math.Vector3
import info.laht.threekt.objects.Mesh
import material.uValue
import org.khronos.webgl.Float32Array
import util.ObjectPool

private const val TERRAIN_SEED: Int = 124
private const val RADIUS: Double = 6.371e6
private const val HEIGHT_SCALE: Double = 1e4
private const val MAX_LOD: Int = 23 // any value up to 28
private const val MAX_ENCODED_LOD: Int = 28 // max LOD able to be encoded

// Distance in tile widths at which a tile subdivides
private const val REL_SUBDIVISION_DIST: Double = 5 * RADIUS // must be > tile w
private const val TILE_POLYGON_WIDTH: Int = 8 // width in polygons of tile
private const val TILE_HEIGHT_ROW_SIZE = TILE_POLYGON_WIDTH + 1
private const val TILE_VERTICES_ROW_SIZE = TILE_HEIGHT_ROW_SIZE + 2
private const val N_TILE_HEIGHTS: Int =
        TILE_HEIGHT_ROW_SIZE * TILE_HEIGHT_ROW_SIZE
private const val N_TILE_VERTICES: Int =
        TILE_VERTICES_ROW_SIZE * TILE_VERTICES_ROW_SIZE

private const val TILE_LIP_BASE_SCALE: Double = 1.0 / TILE_POLYGON_WIDTH
private const val MAX_TILE_DIVISIONS_PER_TIC = 32

private const val SMALL_TEX_CHUNK_SIZE = 6000.0


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

    // Member variables -----------------------------------------------

    override var threeObject: Object3D = Object3D() // nothing special

    val radius = RADIUS
    val faces: Array<Tile> = Array(6) { Tile(this, it) }
    /** Stores tiles no longer in use, but whose geometry can be reused */
    private val tilePile: ObjectPool<Tile> = ObjectPool {
        Tile(this, 0)  // Factory callback.
    }

    var subdivisionCounter = MAX_TILE_DIVISIONS_PER_TIC

    /** Gravitational acceleration of terrestrial objects */
    val gravity: Double = 9.806

    val material = makeMaterial()

    // Init -----------------------------------------------------------

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

    // Actions --------------------------------------------------------

    /**
     * Updates terrain, checking to see which tiles owned by Terrain
     * require subdivision, and which can be recombined.
     * @param tic: Core.Tic
     */
    override fun update(tic: Core.Tic) {
        subdivisionCounter = MAX_TILE_DIVISIONS_PER_TIC
        val sunPos: Double3 = scene!!.sunLight.position
        val sunRelPos = position - sunPos
        (material.unsafeCast<dynamic>()).uniforms.u_dir_light = uValue(Vector3(
                sunRelPos.x, sunRelPos.y, sunRelPos.z
        ))
    }

    /**
     * Cleans up at end of update tic.
     *
     * Performs needed upkeep.
     */
    override fun updateEnd(tic: Core.Tic) {
        tilePile.upkeep()
    }

    // Getters and Setters --------------------------------------------

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
     * Gets Tile identified by quadrants.
     */
    fun getTileFromQuadrants(quadrants: Array<Int>): Tile? {
        var i = 0
        var tile: Tile = faces[i]
        i++
        while (i < quadrants.size) {
            if (!tile.isSubdivided()) {
                return null
            }
            tile = tile.subTile(quadrants[i])
        }
        return tile
    }

    /**
     * Gets height at surface position
     *
     * Returned height is relative to mean surface level.
     */
    @Suppress("UNUSED_PARAMETER") // used in js
    fun heightAtVector(vector: Double3): Double {
        return js("_ter_GetHeight(" +
                "vector.x, vector.y, vector.z, $MAX_LOD)") as Double *
                HEIGHT_SCALE
    }

    /**
     * Gets normal at surface position.
     *
     * Expects input to be normalized.
     */
    fun normalAtVector(v0: Double3, r: Double): Double3 {
        // Hardly a perfect sampling method, but much cheaper in
        // cpu time.
        val v1: Double3 = normalize(Double3(v0.x + r, v0.y, v0.z))
        val v2: Double3 = normalize(Double3(v0.x, v0.y + r, v0.z))
        val v3: Double3 = normalize(Double3(v0.x, v0.y, v0.z + r))

        // Find distance of each point from v0
        val d1 = length(v1 - v0)
        val d2 = length(v2 - v0)
        val d3 = length(v3 - v0)

        // Find sample positions.
        val sample0: Double3  = normalize(v0) * (heightAtVector(v0) + radius)
        val sample1: Double3
        val sample2: Double3
        if (d1 < d2 && d1 < d3) {
            sample1 = v2 * (heightAtVector(v2) + radius)
            sample2 = v3 * (heightAtVector(v3) + radius)
        } else if (d2 < d1 && d2 < d3) {
            sample1 = v1 * (heightAtVector(v1) + radius)
            sample2 = v3 * (heightAtVector(v3) + radius)
        } else {
            sample1 = v1 * (heightAtVector(v1) + radius)
            sample2 = v2 * (heightAtVector(v2) + radius)
        }

        // Get direction from sample0 to sample1 and sample2
        val dir0 = normalize(sample1 - sample0)
        val dir1 = normalize(sample2 - sample0)

        // Get perpendicular vector
        return cross(dir0, dir1) * -1.0
    }

    private fun makeMaterial(): Material {
        val planeMaterial = getTerrainMat(
                Double3(0.7, 0.7, 0.7), 500.0, 120000.0
        )
        //planeMaterial.color = Color(0x3cff00)
        planeMaterial.side = BackSide
        //planeMaterial.metalness = 0.2
        //planeMaterial.roughness = 0.6
        //planeMaterial.wireframe = true // for debugging
        //planeMaterial.flatShading = true
        return planeMaterial
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

            val pos: Double3 = setVertices()
            geometry.computeBoundingSphere()
            geometry.attributes.position.needsUpdate = true
            geometry.attributes.normal.needsUpdate = true
            geometry.attributes.a_tex_pos.needsUpdate = true

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
                    val cubeRelPos: Double3 = cubeRelPosFromHeightIndex(it)
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

            val vertNormals: Array<Double3> = Array(N_TILE_VERTICES) {
                val (heightIndex: Int, _: Boolean) = vertexData(it)
                // Sanity check
                if (heightIndex < 0 || heightIndex >= N_TILE_HEIGHTS) {
                    throw IllegalStateException(
                            "bad height index: $heightIndex. vert: $it")
                }
                val normVec = normalize(cubeRelPosFromHeightIndex(heightIndex))
                val r: Double = relativeWidth / TILE_POLYGON_WIDTH / 4 * RADIUS
                terrain.normalAtVector(normVec, r / RADIUS)
            }

            val relativeCenter: Double3 = vertPositions[N_TILE_VERTICES / 2]
            val geometry = this.geometry ?: throw IllegalStateException(
                    "Tile geometry is null in setGeometry")
            val positionsArray = geometry.getAttribute("position").array
            val normalArray = geometry.getAttribute("normal").array
            val texCoordArray = geometry.getAttribute("a_tex_pos").array
            for (i in 0 until N_TILE_VERTICES) {
                var pos = vertPositions[i]
                pos -= relativeCenter
                val normal = vertNormals[i]

                val vertexStartIndex: Int = i * 3
                positionsArray[vertexStartIndex] = pos.x
                positionsArray[vertexStartIndex + 1] = pos.y
                positionsArray[vertexStartIndex + 2] = pos.z
                normalArray[vertexStartIndex] = normal.x
                normalArray[vertexStartIndex + 1] = normal.y
                normalArray[vertexStartIndex + 2] = normal.z

                val texPos: Double3 = smallTexPos(vertPositions[i])
                texCoordArray[vertexStartIndex] = texPos.x
                texCoordArray[vertexStartIndex + 1] = texPos.y
                texCoordArray[vertexStartIndex + 2] = texPos.z
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
            val geo = PlaneBufferGeometry(1, 1, 10, 10)
            val texPosArr = Float32Array(3 * N_TILE_VERTICES)
            geo.addAttribute(
                    "a_tex_pos", js("new THREE.BufferAttribute(texPosArr, 3)")
            )
            return geo
        }

        val geometry: PlaneBufferGeometry = makeGeometry()
        this.geometry = geometry
        val material: Material = terrain.material
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

    /**
     * Modify passed height array to blend edges with neighbors.
     */
    private fun blendEdges(positions: Array<Double3>) {
        // No point blending at lowest lod.
        if (lod < 2) {
            return
        }

        // Find neighbors
        val above: Tile? = getAboveTile()
        val left: Tile? = getLeftTile()
        val below: Tile? = getBelowTile()
        val right: Tile? = getRightTile()

        var i = 1
        while (i < TILE_HEIGHT_ROW_SIZE) {
            // Blend top edge
            if (above == null) {
                positions[i] = (positions[i - 1] + positions[i + 1]) / 2.0
            }

            // Blend lower edge
            // Blend left edge
            // Blend right edge
            i += 2
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

    fun getAboveTile(): Tile? {
        if (lod < 2) {
            return null
        }

        // Make quadrant copy which will be modified
        val neighborQuadrants: Array<Int> = quadrants.copyOf()

        // Walk upwards until we can switch to a quadrant 'above'
        var i: Int = lod - 1
        var q: Int
        while (i > 0) {
            q = quadrants[i]
            if (q >= 2) {
                if (q == 2) {
                    neighborQuadrants[i] = 1
                } else if (q == 3) {
                    neighborQuadrants[i] = 0
                }
                i++
                while (i < lod) {
                    q = quadrants[i]
                    if (q == 1) {
                        neighborQuadrants[i] = 2
                    } else if (q == 0) {
                        neighborQuadrants[i] = 3
                    }
                    i++
                }
                return terrain.getTileFromQuadrants(neighborQuadrants)
            }
            i--
        }

        // If no tile above this Tile was found, return null.
        return null
    }

    fun getLeftTile(): Tile? {
        if (lod < 2) {
            return null
        }
        return null
    }

    fun getBelowTile(): Tile? {
        if (lod < 2) {
            return null
        }
        return null
    }

    fun getRightTile(): Tile? {
        if (lod < 2) {
            return null
        }
        return null
    }

    fun isSubdivided(): Boolean {
        return subTiles[0] != null
    }

    fun subTile(q: Int): Tile {
        return subTiles[q] ?: throw IllegalStateException("Tile not subdivided")
    }

    fun cubeRelPosFromHeightIndex(i: Int): Double3 {
        val tileRelPos = tilePosFromHeightIndex(i)
        val facePos: Double2 = p1 + tileRelPos * shape
        return facePosTo3d(facePos)
    }

    /**
     * Get texture generation coordinate for vertex.
     *
     * This function produces position vectors that are smaller than
     * the vertex's global coordinate, in order to avoid floating
     * point precision issues on the GPU, which will be using 32bit
     * floats or smaller to handle the vert position when generating
     * simplex noise and other functions that require a value that
     * changes proportionally to the true global coordinate.
     *
     * @param pos World position vector.
     * @return vector that has changes proportional to any changes in
     *          the world position vector. IE, edge-wrapping excepted,
     *          smallTexPos(pos + x) - smallTexPos(pos) == x
     */
    private fun smallTexPos(pos: Double3): Double3 {
        fun mod(a: Double, n: Double): Double {
            var r = (a).rem(n)
            if (r < 0) r += n
            return r
        }
        return abs(Double3(
                mod(pos.x, SMALL_TEX_CHUNK_SIZE * 4),
                mod(pos.y, SMALL_TEX_CHUNK_SIZE * 4),
                mod(pos.z, SMALL_TEX_CHUNK_SIZE * 4)
        ) - Double3(
                SMALL_TEX_CHUNK_SIZE * 2,
                SMALL_TEX_CHUNK_SIZE * 2,
                SMALL_TEX_CHUNK_SIZE * 2
        )) - Double3(
                SMALL_TEX_CHUNK_SIZE,
                SMALL_TEX_CHUNK_SIZE,
                SMALL_TEX_CHUNK_SIZE
        )
    }
}

/**
 * Reconstructs tile quadrants array from passed tile position code.
 */
fun getPositionFromCode(encodedPos: Long): Pair<Int, Array<Int>> {
    // 5b: lod, 3b: face, 2b * LOD: quadrants
    val nQuadrants: Int = (encodedPos and 0x1F).toInt()
    val face: Int = ((encodedPos shr 5) and 0x7).toInt()
    val quadrants: Array<Int> = Array(nQuadrants) {
        i -> ((encodedPos shr 8 + 2 * i) and 0x3).toInt()
    }
    return Pair(face, quadrants)
}
