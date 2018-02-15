package objects

import com.curiouscreature.kotlin.math.Double2
import com.curiouscreature.kotlin.math.Double3
import objects.intf.Positionable
import objects.intf.ThreeOwner

// distance in tile widths at which a tile subdivides
private const val REL_SUBDIVISION_DIST: Double = 3.0


/**
 * Terrain
 */
open class Terrain(id: String=""):
        GameObject("Terrain", id), Positionable
{
    override var position: Double3 = Double3()
}


class Tile(val face: Int, val parent: Tile?=null, val quadrant: Int?=null):
        GameObject(),
        Positionable,
        ThreeOwner
{
    override var position: Double3 = Double3()
    override var threeObject: dynamic = null // todo: set tile obj

    val lod: Int = if (parent == null) 1 else parent.lod + 1
    val shape: Double2 = if (parent != null) parent.shape / 2.0 else
        Double2(2.0, 2.0)
    val subTiles: Array<Tile?> = arrayOfNulls<Tile?>(4)

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
        if (dist / REL_SUBDIVISION_DIST < 1 && subTiles[0] == null) {
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
            tile = Tile(face=face, parent=this, quadrant=i)
            scene!!.add(tile)
            subTiles[i] = tile
        }
        setVisibility(false)
    }

    /**
     * Recombines tile, removing sub-tiles
     */
    private fun recombine() {
        for ((i, tile) in subTiles.withIndex()) {
            scene!!.remove(tile!!)
            subTiles[i] = null
        }
        setVisibility(true)
    }

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
        if (lod > 28) {
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
