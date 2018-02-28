import events.GameEvent

/**
 * Class handling game events either generated locally,
 * or passed by the server.
 *
 * EventHandler is intended to be observable, and observers
 * may be attached by calling .addObserver() with an object extending
 * the EventObserver interface. These observers are then notified when
 * events are received.
 *
 * @see EventObserver
 * @see GameEvent
 */
class EventHandler {

    /**
     * Adds observer to this handler.
     * The passed observer will be notified when an event is received,
     * either from a local object, or the server.
     * @see EventObserver.notify
     * @param observer: EventObserver
     * @return Boolean indicating whether observer was added
     *              successfully or not.
     */
    fun addObserver(observer: EventObserver): Boolean {
        return false
    }

    /**
     * Handles a newly received event, either one created locally, or
     * one passed by the server.
     * @see EventObserver.notify
     * @param event: GameEvent
     */
    fun handleEvent(event: GameEvent) {

    }
}

/**
 * The EventObserver interface denotes a class intended to receive
 * events that describe some action which has taken place in the
 * program.
 */
interface EventObserver {

    /**
     * Function called by EventHandler when an event is received.
     * Returns true to indicate that event should be consumed;
     * not passed on to any other observers.
     * If a response is returned, event will also be consumed, and
     * the response will be sent back to the sender.
     * @see EventHandler.handleEvent
     * @param event: GameEvent
     * @return Boolean or Response
     */
    fun notify(event: GameEvent): dynamic {
        return false
    }

}