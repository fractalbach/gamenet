import com.curiouscreature.kotlin.math.Double3
import info.laht.threekt.math.Color
import info.laht.threekt.renderers.WebGL2Renderer
import info.laht.threekt.renderers.WebGLRenderer
import info.laht.threekt.scenes.Fog
import info.laht.threekt.scenes.FogExp2
import info.laht.threekt.scenes.Scene
import objects.*
import kotlin.browser.window

/**
 * Scene wraps THREE.Scene and contains references to GameObjects
 * specific to the current scene.
 *
 * Keeping all objects and variables specific to a scene contained
 * within a single Scene instance allows easy switches between scenes.
 *
 * Scene instances are responsible for updating their owned GameObjects
 * every game tic.
 *
 * @see GameObject
 */
class Scene(val name: String="Unnamed", var core: Core?=null) {
    // store logger in a companion
    companion object {
        private val logger = Logger.getLogger("Scene")
    }

    private val gameObjects: HashMap<String, GameObject> = HashMap()
    private val removalQueue: ArrayList<GameObject> = ArrayList()
    /** THREE scene which is wrapped by this Scene */
    val threeScene: Scene = info.laht.threekt.scenes.Scene()
    /** THREE renderer that is the default for rendering to view */
    val renderer = WebGLRenderer()
    /** Width of render result */
    val renderWidth: Int = window.innerWidth * 9 / 10
    /** Height of render result */
    val renderHeight: Int = renderWidth * 7 / 10

    // instantiate constant game objects
    /** World Terrain instance - owner of procedural land tiles */
    val terrain: Terrain = Terrain()
    /** Main Camera which is used to see the world from */
    val camera: Camera = objects.FollowCamera()
    /** Overhead mono-directional light source. */
    val sunLight = SunLight("SunLight")
    /**
     * Ambient, omni-present light cast on all objects in scene.
     * Will be adjusted depending on surroundings.
     */
    val ambientLight = AmbientLight("AmbientLight")

    init {
        val r = renderer
        logger.info("Initializing $this")
        // setup renderer
        r.setClearAlpha(1)
        r.setClearColor(0xfffafa, 1)
        js("r.shadowMap.enabled = true;") //enable shadow
        js("r.shadowMap.type = THREE.PCFSoftShadowMap;")
        r.setSize(renderWidth, renderHeight)
        // setup threeScene
        threeScene.fog = Fog()
        threeScene.fog.color = Color(0xf0fff0)
        threeScene.fog.near = 500
        threeScene.fog.far = 120000 // 1e5


        sunLight.position = Double3(1e9, 1e9, 30.0)

        // add constant game objects
        add(terrain)
        add(camera)
        add(sunLight)
        add(ambientLight)

        // test obj
        val mover = TestMover()
        mover.position = Double3(6.0, 0.0, 0.0)
        add(mover)
        (camera as FollowCamera).follow(mover)
    }

    /**
     * Update method; called once per logical tic.
     * Performs any regular updates on scene and
     * in turn calls update on all owned game objects.
     *
     * @param tic: Core.Tic containing information about the current
     *          game tic, such as timestamp and elapsed time.
     * @see Core.update
     * @see GameObject.update
     */
    fun update(tic: Core.Tic) {
        // remove items marked for removal
        removalQueue.forEach { removeHard(it) }
        for (o: GameObject in gameObjects.values) {
            try {
                o.update(tic)
            } catch (e: Exception) {
                logger.error("Error occurred calling $o .update() method.")
                throw e
            }
        }
    }

    /**
     * Renders scene using passed camera, or main scene camera if none
     * is passed.
     * @param camera: THREE.Camera to be used for rendering scene.
     *              Must be a member of the scene.
     */
    fun render(camera: info.laht.threekt.cameras.Camera=this.camera.threeCamera) {
        renderer.render(threeScene, camera)
    }

    /**
     * Adds passed GameObject to scene.
     * Passed GameObject is given a reference to scene.
     * @param gameObject: GameObject to be added.
     * @throws IllegalArgumentException if gameObject is already a
     *              member of the Scene.
     */
    fun add(gameObject: GameObject) {
        logger.fine("Adding $gameObject to $this")
        if (gameObject.id in gameObjects.keys) {
            throw IllegalArgumentException(
                    "$gameObject already a member of $this")
        }
        gameObject.scene = this
        gameObjects[gameObject.id] = gameObject
        threeScene.add(gameObject.threeObject)
        for (childObject in gameObject.childObjects) {
            add(childObject)
        }
    }

    /**
     * Retrieves GameObject owned by this scene, using the
     * object's UUID
     * @param id: UUID String
     * @return GameObject of passed UUID or null, if none was found.
     */
    fun get(id: String): GameObject? {
        return gameObjects[id]
    }

    /**
     * Marks passed game object for removal.
     * GameObject is not actually removed until the beginning of the
     * next logic tic.
     * @param gameObject: GameObject to remove.
     * @throws IllegalArgumentException If GameObject was not a member
     *              of this Scene.
     */
    fun remove(gameObject: GameObject) {
        logger.fine("Removing $gameObject from $this")
        if (gameObject.id !in gameObjects.keys) {
            throw IllegalArgumentException(
                    "$gameObject not a member of $this")
        }
        removalQueue.add(gameObject)
    }

    /**
     * This function is what actually removes an object from
     * scene. It is not to be called directly from outside Scene.
     * Instead, GameObjects will be queued for removal by calling the
     * public remove() method, and then this method will be called
     * between updates.
     */
    private fun removeHard(gameObject: GameObject) {
        gameObject.scene = null
        gameObjects.remove(gameObject.id)
        threeScene.remove(gameObject.threeObject)
        for (childObject in gameObject.childObjects) {
            remove(childObject)
        }
    }

    /** @suppress Gives String representation */
    override fun toString(): String = "Scene[$name]"
}

// Notes: If any more kinds of scenes need to be added (that would
// require different renderer settings, etc) it would probably be best
// if Scene were made more generic, and the settings applied wherever
// Scene was instantiated, or else in a Scene subclass.
