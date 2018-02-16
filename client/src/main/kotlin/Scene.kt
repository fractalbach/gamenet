import com.curiouscreature.kotlin.math.Double3
import info.laht.threekt.math.Color
import info.laht.threekt.math.Vector3
import info.laht.threekt.renderers.WebGLRenderer
import info.laht.threekt.scenes.FogExp2
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
 */
class Scene(val name: String="Unnamed", var core: Core?=null) {
    companion object {
        val logger = Logger.getLogger("Scene")
    }

    private val gameObjects: HashMap<String, GameObject> = HashMap()
    val threeScene = info.laht.threekt.scenes.Scene()
    val renderer = WebGLRenderer()
    val renderWidth: Int = window.innerWidth * 9 / 10
    val renderHeight: Int = window.innerHeight * 9 / 10

    // instantiate constant game objects
    //val terrain: Terrain = Terrain()
    val camera: Camera = objects.Camera()
    val sunLight = SunLight("SunLight")

    init {
        logger.info("Initializing $this")
        // setup renderer
        renderer.setClearAlpha(1)
        renderer.setClearColor(0xfffafa, 1)
        //js("this.renderer.shadowMap.enabled = true") //enable shadow
        //js("this.renderer.shadowMap.type = PCFSoftShadowMap()")
        renderer.setSize(renderWidth, renderHeight)
        // setup threeScene
        threeScene.fog = FogExp2(Color(0xf0fff0), 0.1 )

        camera.position = Double3(0.0, 2.5, 6.5)
        sunLight.position = Double3(0.0, 100.0, 30.0)

        // add constant game objects
        //add(terrain)
        add(camera)
        add(sunLight)
        add(TestCube("TestCube"))
    }

    /**
     * Update method; called once per logical tic.
     * Performs any regular updates on scene and
     * in turn calls update on all owned game objects.
     */
    fun update() {
        for (o: GameObject in gameObjects.values) {
            o.update()
        }
    }

    /**
     * Renders scene using
     */
    fun render(camera: info.laht.threekt.cameras.Camera=this.camera.threeCamera) {
        renderer.render(threeScene, camera)
    }

    /**
     * Adds passed GameObject to scene.
     * Passed GameObject is given a reference to scene.
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

    fun get(id: String): GameObject? {
        return gameObjects[id]
    }

    fun remove(gameObject: GameObject) {
        // todo
    }

    override fun toString(): String {
        return "Scene[$name]"
    }
}

// Notes: If any more kinds of scenes need to be added (that would
// require different renderer settings, etc) it would probably be best
// if Scene were made more generic, and the settings applied wherever
// Scene was instantiated, or else in a Scene subclass.
