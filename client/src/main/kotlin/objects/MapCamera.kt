package objects

import Core
import com.curiouscreature.kotlin.math.Double2
import com.curiouscreature.kotlin.math.Double3
import com.curiouscreature.kotlin.math.clamp
import com.curiouscreature.kotlin.math.radians
import info.laht.threekt.cameras.PerspectiveCamera
import info.laht.threekt.math.Euler
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin

private const val LAT_COEF: Double = 0.00000004  // At 10km alt.
private const val LON_COEF: Double = 0.00000004  // At 10km alt.
private const val ALT_COEF: Double = 0.02
private const val MIN_ALT: Double = 10_000.0
private const val MAX_ALT: Double = 10_000_000.0


/**
 * Camera that views the map
 */
open class MapCamera(name: String="", id: String=""): Camera(name, id) {
    var followed: Terrain? = null

    var lat: Double = 0.0
    var lon: Double = 0.0
    var alt: Double = 100_000.0

    /**
     * Set Terrain (planet) for MapCamera to focus on.
     */
    fun focus(world: Terrain) {
        world.threeObject.add(threeObject)
        this.followed = world
        position = findPosition()
        rotation = findRotation()
        logger.debug("$this (pos: $position) now focussed on " +
                "$world (pos: ${world.position})")
    }

    /**
     * Reads inputs from user, updates lat, lon, and camera
     * position / rotation.
     */
    override fun update(tic: Core.Tic) {
        super.update(tic)

        val traverse: Double2 = tic.core.input.mouseMotion
        var moved = false
        if (traverse.x != 0.0) {
            val latMod: Double = min(1.0 / cos(radians(lat)), 100.0)
            lon += -traverse.x * LON_COEF * latMod * alt
            lon = wrap(lon, -180.0, 180.0)
            moved = true
        }
        if (traverse.y != 0.0) {
            lat += traverse.y * LAT_COEF * alt
            lat = clamp(lat, -90.0, 90.0)
            moved = true
        }
        if (tic.core.input.cmdActive(InputHandler.Command.ZOOM_OUT)) {
            alt += alt * ALT_COEF
            alt = max(alt, MIN_ALT)
            moved = true
        }
        if (tic.core.input.cmdActive(InputHandler.Command.ZOOM_IN)) {
            alt -= alt * ALT_COEF
            alt = min(alt, MAX_ALT)
            moved = true
        }

        if (moved) {
            position = findPosition()
            rotation = findRotation()

            // Update view frustum
            val perspectiveCamera = threeCamera as PerspectiveCamera
            perspectiveCamera.far = alt + followed!!.radius
            perspectiveCamera.near = max(alt * 0.25, alt - 20_000.0)
            perspectiveCamera.updateProjectionMatrix()
        }

        // Display debug info
        tic.core.debugInfo["Map Lat"] = lat.toString()
        tic.core.debugInfo["Map Lon"] = lon.toString()
        tic.core.debugInfo["Map Alt"] = alt.toString()
    }

    /**
     * Calculates position relative to parent from lat, lon, and alt.
     */
    private fun findPosition(): Double3 {
        val r = followed!!.radius + alt
        val rLat = radians(lat)
        val rLon = radians(lon)
        return Double3(
                x=r * cos(rLat) * cos(rLon),
                y=r * cos(rLat) * sin(rLon),
                z=r * sin(rLat)
        )
    }

    /**
     * Calculates rotation relative to parent from lat, lon, and alt.
     */
    private fun findRotation(): Euler {
        return Euler(
                y=radians(0.0),
                z=radians(lon + 90.0),
                x=radians(-lat + 90.0),
                order = "ZYX"
        )
    }

    private fun wrap(x: Double, min: Double, max: Double): Double {
        val range: Double = max - min
        return when {
            x < min -> x + range
            x > max -> x - range
            else -> x
        }
    }
}
