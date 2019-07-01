import com.curiouscreature.kotlin.math.Double3
import objects.FollowCamera
import objects.TestMover

@Suppress("unused")  // Called from js at program start.
@JsName("simpleInit")
fun simpleInit(core: Core) {
    // test obj
    val mover = TestMover()
    mover.position = Double3(6.0, 0.0, 0.0)
    val scene: Scene = core.scene
    scene.add(mover)
    (scene.camera as FollowCamera).follow(mover)
}
