package exception

/**
 * Thrown when the state of the document is not as expected.
 * This indicates an unrecoverable flaw in resources or code.
 */
class DocumentError(
        override var message: String,
        override var cause: Throwable? = null) : Exception(message, cause)


/**
 * Thrown when a C function returns a value indicating that an
 * exception has occurred during call of the function.
 */
class CException(
        override var message: String,
        override var cause: Throwable? = null) : Exception(message, cause)


/**
 * Root exception thrown by exceptions relating to game logic, timing,
 * physics, etc.
 */
class GameException(
        override var message: String,
        override var cause: Throwable? = null) : Exception(message, cause)