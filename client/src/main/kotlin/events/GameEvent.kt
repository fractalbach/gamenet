package events


/**
 * Generic event storing only basic fields.
 * Intended to be extended in order to provide specialized fields.
 */
open class GameEvent(
        val sourceId: String, val sourceType: String, val eventType: String) {

    /** UUID for identifying this specific event */
    val id: String = js("uuid()") as String // generate event UUID

    init {
        // validate inputs
        if (sourceId.isEmpty()) {
            throw IllegalArgumentException(
                    "GameEvent sourceId passed at initialization was empty")
        }
        if (sourceType.isEmpty()) {
            throw IllegalArgumentException(
                    "GameEvent constructor was passed an empty sourceType")
        }
        if (eventType.isEmpty()) {
            throw IllegalArgumentException(
                    "GameEvent constructor was passed an empty eventType str")
        }
    }
}


/**
 * A Response is a GameEvent that is sent in response to an earlier
 * received GameEvent, and will usually contain information expected
 * by the sender of the original event.
 */
open class Response(sourceId: String,
        sourceType: String, eventType: String, event: GameEvent):
    GameEvent(sourceId, sourceType, eventType)
{
    /** Id of the GameEvent that is being responded to */
    val responseId: String = event.id
}