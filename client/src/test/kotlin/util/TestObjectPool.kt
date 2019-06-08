package util

import kotlin.test.*

class TestObjectPool {

    class TestObject (val i: Int)

    @Test
    fun testObjectPoolRecycling() {
        var counter = 0
        val pool = ObjectPool{ TestObject(counter++) }

        val used = ArrayList<TestObject>()
        for (i in 0..9) {
            used.add(pool.get())
        }
        for (i in 0..9) {
            for (j in 0..4) {
                pool.recycle(used.removeAt(used.lastIndex))
            }
            pool.upkeep()
            for (j in 0..4) {
                used.add(pool.get())
            }
            pool.upkeep()
        }
        assertEquals(10, counter)
        for (i in 0..8) {
            pool.recycle(used.removeAt(used.lastIndex))
        }
    }
}
