import com.curiouscreature.kotlin.math.Double2
import objects.Terrain
import objects.Tile
import kotlin.browser.window
import kotlin.test.*

class TestTerrain {

    // ----------------------------------------------------------------
    // POS CODE TESTS
    // ----------------------------------------------------------------

    @Test
    fun testTileReturnsPosCode0() {
        if (js("Module.ready") != true) {
            console.log("waiting")
            js("wait()")
        }
        val terrain: Terrain = Terrain()
        val tile: Tile = Tile(terrain=terrain, face=0)

        assertEquals(1L,tile.getPositionCode())
    }

    @Test
    fun testTileReturnsPosCode1() {
        if (js("Module.ready") != true) {
            console.log("waiting")
            js("wait()")
        }
        val terrain: Terrain = Terrain()
        val tile: Tile = Tile(terrain=terrain, face=1)

        assertEquals(33L,tile.getPositionCode())
    }

    @Test
    fun testTileReturnsPosCode2() {
        if (js("Module.ready") != true) {
            console.log("waiting")
            js("wait()")
        }
        val terrain: Terrain = Terrain()
        val tile: Tile = Tile(terrain=terrain, face=2)

        assertEquals(65L,tile.getPositionCode())
    }

    @Test
    fun testTileReturnsPosCode3() {
        if (js("Module.ready") != true) {
            console.log("waiting")
            js("wait()")
        }
        val terrain: Terrain = Terrain()
        val tile: Tile = Tile(terrain=terrain, face=3)

        assertEquals(97L,tile.getPositionCode())
    }

    @Test
    fun testTileReturnsPosCode4() {
        if (js("Module.ready") != true) {
            console.log("waiting")
            js("wait()")
        }
        val terrain: Terrain = Terrain()
        val tile: Tile = Tile(terrain=terrain, face=4)

        assertEquals(129L,tile.getPositionCode())
    }

    @Test
    fun testTileReturnsPosCode5() {
        if (js("Module.ready") != true) {
            console.log("waiting")
            js("wait()")
        }
        val terrain: Terrain = Terrain()
        val tile: Tile = Tile(terrain=terrain, face=5)

        assertEquals(161L,tile.getPositionCode())
    }

    // ----------------------------------------------------------------
    // Transformations
    // ----------------------------------------------------------------

    @Test
    fun testTileIndexZeroHasTilePositionZeroZero() {
        assertEquals(Double2(0.0, 0.0),
                Tile.Companion.tilePosFromVertIndex(0))
    }

    @Test
    fun testLastTileIndex80HasTilePositionOneOne() {
        assertEquals(Double2(1.0, 1.0),
                Tile.Companion.tilePosFromVertIndex(80))
    }

    @Test
    fun testTileIndex40HasTileCenterPosition() {
        assertEquals(Double2(0.5, 0.5),
                Tile.Companion.tilePosFromVertIndex(40))
    }

    @Test
    fun testTileIndex8HasTileLowRightPosition() {
        assertEquals(Double2(1.0, 0.0),
                Tile.Companion.tilePosFromVertIndex(8))
    }

    @Test
    fun testTileIndex72HasTileHighLeftPosition() {
        assertEquals(Double2(0.0, 1.0),
                Tile.Companion.tilePosFromVertIndex(72))
    }
}