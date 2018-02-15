import objects.GameObject

class TestScene {

    fun testSceneGivesReferenceToSelfToAddedGameObject() {
        val scene: Scene = Scene("TestScene")
        val gameObject: GameObject = GameObject()
        scene.add(gameObject)

        //assertTrue(gameObject.scene == scene)

    }
}