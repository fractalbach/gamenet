import com.curiouscreature.kotlin.math.Double2
import objects.Terrain
import objects.Tile
import kotlin.test.*

class TestTerrain {

    // ----------------------------------------------------------------
    // Transformations
    // ----------------------------------------------------------------

    @Test
    fun testTileIndexZeroHasTilePositionZeroZero() {
        assertEquals(Double2(0.0, 0.0),
                Tile.Companion.tilePosFromHeightIndex(0))
    }

    @Test
    fun testLastTileIndex80HasTilePositionOneOne() {
        assertEquals(Double2(1.0, 1.0),
                Tile.Companion.tilePosFromHeightIndex(80))
    }

    @Test
    fun testTileIndex40HasTileCenterPosition() {
        assertEquals(Double2(0.5, 0.5),
                Tile.Companion.tilePosFromHeightIndex(40))
    }

    @Test
    fun testTileIndex8HasTileLowRightPosition() {
        assertEquals(Double2(1.0, 0.0),
                Tile.Companion.tilePosFromHeightIndex(8))
    }

    @Test
    fun testTileIndex72HasTileHighLeftPosition() {
        assertEquals(Double2(0.0, 1.0),
                Tile.Companion.tilePosFromHeightIndex(72))
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectIndexZeroData() {
        val (hIndex, lip: Boolean) = Tile.Companion.vertexData(0)
        assertEquals(0, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectLowerEdgeData() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(4)
        assertEquals(3, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectUpperEdgeData() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(117)
        assertEquals(78, hIndex, "input 117 should produce 78")
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectLeftEdgeData() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(11)
        assertEquals(0, hIndex, "input 11 should produce 0")
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectLeftEdgeData2() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(33)
        assertEquals(18, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectRightEdgeData1() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(32)
        assertEquals(17, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectRightEdgeData2() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(109)
        assertEquals(80, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectRightEdgeData3() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(98)
        assertEquals(71, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectUpperRightCorner() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(120)
        assertEquals(80, hIndex)
        assertEquals(true, lip)
    }

    @Test
    fun testHeightIndexFromVertexIndexGivesCorrectPositionNotOnEdge() {
        val (hIndex: Int, lip: Boolean) = Tile.Companion.vertexData(30)
        assertEquals(16, hIndex)
        assertFalse(lip)
    }

    @Test
    fun testHeightFromIndexThrowsIllegalArgumentWhenPassedIndexUnderRange() {
        assertFailsWith(IllegalArgumentException::class) {
            Tile.Companion.vertexData(-1)
        }
    }

    @Test
    fun testHeightFromIndexThrowsIllegalArgumentWhenPassedIndexOverRange() {
        assertFailsWith(IllegalArgumentException::class) {
            Tile.Companion.vertexData(121)
        }
    }
}