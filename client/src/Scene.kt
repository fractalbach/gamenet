import objects.GameObject
import objects.StaticCamera
import objects.Terrain
import objects.intf.Camera
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
    private val gameObjects: HashMap<String, GameObject> = HashMap()
    val threeScene = js("new THREE.Scene()")!!
    val renderer = js("new THREE.WebGLRenderer({alpha:true})")!!
    val renderWidth: Int = window.innerWidth
    val renderHeight: Int = window.innerHeight

    // instantiate constant game objects
    val terrain: Terrain = Terrain()
    val camera: Camera = StaticCamera()

    init {
        // setup renderer
        renderer.setClearColor(0xfffafa, 1)
        renderer.shadowMap.enabled = true //enable shadow
        renderer.shadowMap.type = js("THREE.PCFSoftShadowMap")!!
        renderer.setSize(renderWidth, renderHeight)
        // setup threeScene
        threeScene.fog = js("new THREE.FogExp2( 0xf0fff0, 0.14 )")!!
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
     * Adds passed GameObject to scene.
     * Passed GameObject is given a reference to scene.
     */
    fun add(gameObject: GameObject) {
        if (gameObject.id in gameObjects.keys) {
            throw IllegalArgumentException(
                    "$gameObject already a member of $this")
        }
        gameObject.scene = this
        gameObjects[gameObject.id] = gameObject
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
