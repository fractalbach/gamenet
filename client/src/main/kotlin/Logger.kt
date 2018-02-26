
enum class LogLevel(val i: Int)
    {FINE(0), DEBUG(1), INFO(2), WARN(3), ERROR(4), WTF(5)}


class Logger private constructor(val name: String) {
    companion object {
        private val loggers = HashMap<String, Logger>()
        private val logger = getLogger("Logger")
        var logLevel: LogLevel = LogLevel.INFO
            set(level) {
                field = level
                logger.info("Log level changed to: ${logLevel.name}")
            }

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

    fun log(level: LogLevel, msg: String, e: Exception?=null) {
        if (level.i < logLevel.i) {
            return
        }
        println("[${level.name.padEnd(5)}] $name:  $msg")
        if (e != null) {
            println(e)
        }
    }

    fun fine(msg: String, e: Exception?=null) {
        log(LogLevel.FINE, msg, e)
    }

    fun debug(msg: String, e: Exception?=null) {
        log(LogLevel.DEBUG, msg, e)
    }

    fun info(msg: String, e: Exception?=null) {
        log(LogLevel.INFO, msg, e)
    }

    fun warn(msg: String, e: Exception?=null) {
        log(LogLevel.WARN, msg, e)
    }

    fun error(msg: String, e: Exception?=null) {
        log(LogLevel.ERROR, msg, e)
    }

    fun wtf(msg: String, e: Exception?=null) {
        log(LogLevel.WTF, msg, e)
    }
}
