/**
 * This file contains the startup function and base class for
 * A WebWorker which handles procedural generation of objects which
 * are to then be passed on to the main loop.
 */


class ProceduralWorker {

    /**
     * Method handling receipt of message from main loop.
     * This will usually be a message informing the procedural worker
     * of a new global position, and requesting any new objects
     * that should be added to the scene.
     */
    fun onMessage(msg: dynamic) {

    }
}


/**
 * This function is called on load and starts a ProceduralWorker.
 */
@JsName("startProceduralWorker")
fun startProceduralWorker(args: Array<String>) {
    // todo
}