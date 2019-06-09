package util

import Logger.Companion.getLogger

/**
 * Class that maintains a pool of objects, to
 * minimize initialization cost.
 */
class ObjectPool<T> (val factory: () -> T){
    companion object {
        private val logger = getLogger("ObjectPool")
    }

    private val pile: ArrayList<T> = ArrayList()
    private val recycled: ArrayList<T> = ArrayList()
    private var nExisting: Int = 0

    /**
     * Get an object of type T.
     *
     * If objects are available in the pool, one will be returned.
     * If no objects are currently unused, one will be created.
     */
    fun get(): T {
        val o: T
        if (pile.isEmpty()) {
            o = factory()
            nExisting++
        } else {
            o = pile.removeAt(pile.lastIndex)
        }
        return o
    }

    /**
     * Add object to the pool for re-use.
     *
     * The object will not actually be able to be re-used until
     * upkeep() is called.
     *
     * @see upkeep
     */
    fun recycle(o: T) {
        recycled.add(o)
    }

    /**
     * Move items from recycle pile to ready pile.
     *
     * If the number of unused exceeds the number of used, some will
     * be freed.
     */
    fun upkeep() {
        while (recycled.isNotEmpty()) {
            val o: T = recycled.removeAt(recycled.lastIndex)
            if (pile.size < nExisting / 2) {  // If more than half are unused...
                pile.add(o)
            } else {
                nExisting--
            }
        }
    }
}
