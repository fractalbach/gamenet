package exception

/**
 * Thrown when the state of the document is not as expected.
 * This indicates an unrecoverable flaw in resources or code.
 */
class DocumentError(override var message: String): Exception(message)

/**
 * Thrown when a C function returns a value indicating that an
 * exception has occurred during call of the function.
 */
class CException(override var message: String): Exception(message)
