import objects.MapCamera

fun mapViewInit(core: Core) {
    val camera = MapCamera()
    val scene: Scene = core.scene
    scene.remove(scene.camera)
    scene.camera = camera
    scene.add(camera)
    camera.focus(scene.terrain)
}
