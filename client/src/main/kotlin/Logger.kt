
/** @suppress Denotes different levels of logging priority */
enum class LogLevel(val i: Int)
    {FINE(0), DEBUG(1), INFO(2), WARN(3), ERROR(4), WTF(5)}


/**
 * Logger provides a unified way to log events that take place
 * during execution, at varying levels of importance.
 *
 * This Logger is intended to resemble a minimalist version of Loggers
 */
class Logger private constructor(val name: String) {
    companion object {
        private val loggers = HashMap<String, Logger>()
        private val logger = getLogger("Logger")
        /**
         * level at which events are being logged.
         * Events with a lower level are simply ignored.
         */
        var logLevel: LogLevel = LogLevel.FINE
            set(level) {
                field = level
                logger.info("Log level changed to: ${logLevel.name}")
            }

        /**
         * Gets logger of passed name, or creates it if it does
         * not exist.
         */
        fun getLogger(name: String): Logger {
            lateinit var logger: Logger
            if (name in loggers.keys) {
                logger = loggers[name]!!
            } else {
                logger = Logger(name)
                loggers[name] = logger
            }
            return logger
        }
    }

    /**
     * Generic log method; Logs a message of passed level and message.
     * @param level: LogLevel (FINE, DEBUG, INFO, WARN, ERROR, WTF)
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun log(level: LogLevel, msg: String, e: Exception?=null) {
        if (level.i < logLevel.i) {
            return
        }
        println("[${level.name.padEnd(5)}] $name:  $msg")
        if (e != null) {
            println(e)
        }
    }

    /**
     * Logs a fine message.
     * This should be used for lowest level, high volume messages that
     * often simply provide information about where execution is.
     * Analogous to 'trace' messages in some Loggers.
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun fine(msg: String, e: Exception?=null) {
        log(LogLevel.FINE, msg, e)
    }

    /**
     * Logs a debug message.
     * This should be used to provide low-level information about
     * things that have taken place that are significant only to the
     * internal workings of the program.
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun debug(msg: String, e: Exception?=null) {
        log(LogLevel.DEBUG, msg, e)
    }

    /**
     * Logs an info message.
     * This should be used to provide information about actions that
     * have taken place which are relevant to the overall purpose or
     * goal of the program. (IE: things that would matter to the user,
     * not just developers)
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun info(msg: String, e: Exception?=null) {
        log(LogLevel.INFO, msg, e)
    }

    /**
     * Logs a warning message.
     * This should be used to provide information about things that are
     * oddities in the program execution, and may cause unintended
     * effects in the program.
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun warn(msg: String, e: Exception?=null) {
        log(LogLevel.WARN, msg, e)
    }

    /**
     * Logs an error message.
     * This should be used for messages that provide information about
     * an event that has caused the program to be unable to operate
     * as intended, and is not expected to be recoverable.
     * Info is often the default log level.
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun error(msg: String, e: Exception?=null) {
        log(LogLevel.ERROR, msg, e)
    }

    /**
     * Logs a wtf message.
     * This should be used for errors that should not actually
     * be possible.
     * @param msg: String message to be logged.
     * @param e: Exception that is to be logged with message.
     */
    fun wtf(msg: String, e: Exception?=null) {
        log(LogLevel.WTF, msg, e)
    }
}
