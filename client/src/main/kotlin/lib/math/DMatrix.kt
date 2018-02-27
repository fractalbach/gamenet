/*
 * Copyright (C) 2017 Romain Guy
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *       http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 *
 * library written by Romain Guy
 *
 * library modified by TryExceptElse to offer both float32 and float64
 * operations.
 */

package com.curiouscreature.kotlin.math

import kotlin.math.*

enum class DMatrixColumn {
    X, Y, Z, W
}

data class DMat2(
        var x: Double2 = Double2(x = 1.0),
        var y: Double2 = Double2(y = 1.0)) {
    constructor(m: DMat2) : this(m.x.copy(), m.y.copy())

    companion object {
        fun of(vararg a: Double): DMat2 {
            require(a.size >= 4)
            return DMat2(
                    Double2(a[0], a[2]),
                    Double2(a[1], a[3])
            )
        }

        fun identity() = DMat2()
    }

    operator fun get(column: Int) = when(column) {
        0 -> x
        1 -> y
        else -> throw IllegalArgumentException("column must be in 0..1")
    }
    operator fun get(column: Int, row: Int) = get(column)[row]

    operator fun get(column: MatrixColumn) = when(column) {
        MatrixColumn.X -> x
        MatrixColumn.Y -> y
        else -> throw IllegalArgumentException("column must be X or Y")
    }
    operator fun get(column: MatrixColumn, row: Int) = get(column)[row]

    operator fun invoke(row: Int, column: Int) = get(column - 1)[row - 1]
    operator fun invoke(row: Int, column: Int, v: Double) = set(column - 1, row - 1, v)

    operator fun set(column: Int, v: Double2) {
        this[column].xy = v
    }
    operator fun set(column: Int, row: Int, v: Double) {
        this[column][row] = v
    }

    operator fun unaryMinus() = DMat2(-x, -y)
    operator fun inc(): DMat2 {
        x++
        y++
        return this
    }
    operator fun dec(): DMat2 {
        x--
        y--
        return this
    }

    operator fun plus(v: Double) = DMat2(x + v, y + v)
    operator fun minus(v: Double) = DMat2(x - v, y - v)
    operator fun times(v: Double) = DMat2(x * v, y * v)
    operator fun div(v: Double) = DMat2(x / v, y / v)

    operator fun times(m: DMat2): DMat2 {
        val t = transpose(this)
        return DMat2(
                Double2(dot(t.x, m.x), dot(t.y, m.x)),
                Double2(dot(t.x, m.y), dot(t.y, m.y))
        )
    }

    operator fun times(v: Double2): Double2 {
        val t = transpose(this)
        return Double2(dot(t.x, v), dot(t.y, v))
    }

    fun toDoubleArray() = doubleArrayOf(
            x.x, y.x,
            x.y, y.y
    )

    override fun toString(): String {
        return """
            |${x.x} ${y.x}|
            |${x.y} ${y.y}|
            """.trimIndent()
    }

}

data class DMat3(
        var x: Double3 = Double3(x = 1.0),
        var y: Double3 = Double3(y = 1.0),
        var z: Double3 = Double3(z = 1.0)) {
    constructor(m: DMat3) : this(m.x.copy(), m.y.copy(), m.z.copy())

    companion object {
        fun of(vararg a: Double): DMat3 {
            require(a.size >= 9)
            return DMat3(
                    Double3(a[0], a[3], a[6]),
                    Double3(a[1], a[4], a[7]),
                    Double3(a[2], a[5], a[8])
            )
        }

        fun identity() = DMat3()
    }

    operator fun get(column: Int) = when(column) {
        0 -> x
        1 -> y
        2 -> z
        else -> throw IllegalArgumentException("column must be in 0..2")
    }
    operator fun get(column: Int, row: Int) = get(column)[row]

    operator fun get(column: MatrixColumn) = when(column) {
        MatrixColumn.X -> x
        MatrixColumn.Y -> y
        MatrixColumn.Z -> z
        else -> throw IllegalArgumentException("column must be X, Y or Z")
    }
    operator fun get(column: MatrixColumn, row: Int) = get(column)[row]

    operator fun invoke(row: Int, column: Int) = get(column - 1)[row - 1]
    operator fun invoke(row: Int, column: Int, v: Double) = set(column - 1, row - 1, v)

    operator fun set(column: Int, v: Double3) {
        this[column].xyz = v
    }
    operator fun set(column: Int, row: Int, v: Double) {
        this[column][row] = v
    }

    operator fun unaryMinus() = DMat3(-x, -y, -z)
    operator fun inc(): DMat3 {
        x++
        y++
        z++
        return this
    }
    operator fun dec(): DMat3 {
        x--
        y--
        z--
        return this
    }

    operator fun plus(v: Double) = DMat3(x + v, y + v, z + v)
    operator fun minus(v: Double) = DMat3(x - v, y - v, z - v)
    operator fun times(v: Double) = DMat3(x * v, y * v, z * v)
    operator fun div(v: Double) = DMat3(x / v, y / v, z / v)

    operator fun times(m: DMat3): DMat3 {
        val t = transpose(this)
        return DMat3(
                Double3(dot(t.x, m.x), dot(t.y, m.x), dot(t.z, m.x)),
                Double3(dot(t.x, m.y), dot(t.y, m.y), dot(t.z, m.y)),
                Double3(dot(t.x, m.z), dot(t.y, m.z), dot(t.z, m.z))
        )
    }

    operator fun times(v: Double3): Double3 {
        val t = transpose(this)
        return Double3(dot(t.x, v), dot(t.y, v), dot(t.z, v))
    }

    fun toDoubleArray() = doubleArrayOf(
            x.x, y.x, z.x,
            x.y, y.y, z.y,
            x.z, y.z, z.z
    )

    override fun toString(): String {
        return """
            |${x.x} ${y.x} ${z.x}|
            |${x.y} ${y.y} ${z.y}|
            |${x.z} ${y.z} ${z.z}|
            """.trimIndent()
    }
}

data class DMat4(
        var x: Double4 = Double4(x = 1.0),
        var y: Double4 = Double4(y = 1.0),
        var z: Double4 = Double4(z = 1.0),
        var w: Double4 = Double4(w = 1.0)) {
    constructor(right: Double3, up: Double3, forward: Double3, position: Double3 = Double3()) :
            this(Double4(right), Double4(up), Double4(forward), Double4(position, 1.0))
    constructor(m: DMat4) : this(m.x.copy(), m.y.copy(), m.z.copy(), m.w.copy())

    companion object {
        fun of(vararg a: Double): DMat4 {
            require(a.size >= 16)
            return DMat4(
                    Double4(a[0], a[4], a[8],  a[12]),
                    Double4(a[1], a[5], a[9],  a[13]),
                    Double4(a[2], a[6], a[10], a[14]),
                    Double4(a[3], a[7], a[11], a[15])
            )
        }

        fun identity() = DMat4()
    }

    inline var right: Double3
        get() = x.xyz
        set(value) {
            x.xyz = value
        }
    inline var up: Double3
        get() = y.xyz
        set(value) {
            y.xyz = value
        }
    inline var forward: Double3
        get() = z.xyz
        set(value) {
            z.xyz = value
        }
    inline var position: Double3
        get() = w.xyz
        set(value) {
            w.xyz = value
        }

    inline val scale: Double3
        get() = Double3(length(x.xyz), length(y.xyz), length(z.xyz))
    inline val translation: Double3
        get() = w.xyz
    val rotation: Double3
        get() {
            val x = normalize(right)
            val y = normalize(up)
            val z = normalize(forward)

            return when {
                z.y <= -1.0 -> Double3(degrees(-HALF_PI), 0.0, degrees(atan2( x.z,  y.z)))
                z.y >=  1.0 -> Double3(degrees(HALF_PI), 0.0, degrees(atan2(-x.z, -y.z)))
                else -> Double3(
                        degrees(-asin(z.y)), degrees(-atan2(z.x, z.z)), degrees(atan2( x.y,  y.y)))
            }
        }

    inline val upperLeft: DMat3
        get() = DMat3(x.xyz, y.xyz, z.xyz)

    operator fun get(column: Int) = when(column) {
        0 -> x
        1 -> y
        2 -> z
        3 -> w
        else -> throw IllegalArgumentException("column must be in 0..3")
    }
    operator fun get(column: Int, row: Int) = get(column)[row]

    operator fun get(column: MatrixColumn) = when(column) {
        MatrixColumn.X -> x
        MatrixColumn.Y -> y
        MatrixColumn.Z -> z
        MatrixColumn.W -> w
    }
    operator fun get(column: MatrixColumn, row: Int) = get(column)[row]

    operator fun invoke(row: Int, column: Int) = get(column - 1)[row - 1]
    operator fun invoke(row: Int, column: Int, v: Double) = set(column - 1, row - 1, v)

    operator fun set(column: Int, v: Double4) {
        this[column].xyzw = v
    }
    operator fun set(column: Int, row: Int, v: Double) {
        this[column][row] = v
    }

    operator fun unaryMinus() = DMat4(-x, -y, -z, -w)
    operator fun inc(): DMat4 {
        x++
        y++
        z++
        w++
        return this
    }
    operator fun dec(): DMat4 {
        x--
        y--
        z--
        w--
        return this
    }

    operator fun plus(v: Double) = DMat4(x + v, y + v, z + v, w + v)
    operator fun minus(v: Double) = DMat4(x - v, y - v, z - v, w - v)
    operator fun times(v: Double) = DMat4(x * v, y * v, z * v, w * v)
    operator fun div(v: Double) = DMat4(x / v, y / v, z / v, w / v)

    operator fun times(m: DMat4): DMat4 {
        val t = transpose(this)
        return DMat4(
                Double4(dot(t.x, m.x), dot(t.y, m.x), dot(t.z, m.x), dot(t.w, m.x)),
                Double4(dot(t.x, m.y), dot(t.y, m.y), dot(t.z, m.y), dot(t.w, m.y)),
                Double4(dot(t.x, m.z), dot(t.y, m.z), dot(t.z, m.z), dot(t.w, m.z)),
                Double4(dot(t.x, m.w), dot(t.y, m.w), dot(t.z, m.w), dot(t.w, m.w))
        )
    }

    operator fun times(v: Double4): Double4 {
        val t = transpose(this)
        return Double4(dot(t.x, v), dot(t.y, v), dot(t.z, v), dot(t.w, v))
    }

    fun toDoubleArray() = doubleArrayOf(
            x.x, y.x, z.x, w.x,
            x.y, y.y, z.y, w.y,
            x.z, y.z, z.z, w.z,
            x.w, y.w, z.w, w.w
    )

    override fun toString(): String {
        return """
            |${x.x} ${y.x} ${z.x} ${w.x}|
            |${x.y} ${y.y} ${z.y} ${w.y}|
            |${x.z} ${y.z} ${z.z} ${w.z}|
            |${x.w} ${y.w} ${z.w} ${w.w}|
            """.trimIndent()
    }
}

fun transpose(m: DMat2) = DMat2(
        Double2(m.x.x, m.y.x),
        Double2(m.x.y, m.y.y)
)

fun transpose(m: DMat3) = DMat3(
        Double3(m.x.x, m.y.x, m.z.x),
        Double3(m.x.y, m.y.y, m.z.y),
        Double3(m.x.z, m.y.z, m.z.z)
)
fun inverse(m: DMat3): DMat3 {
    val a = m.x.x
    val b = m.x.y
    val c = m.x.z
    val d = m.y.x
    val e = m.y.y
    val f = m.y.z
    val g = m.z.x
    val h = m.z.y
    val i = m.z.z

    val A = e * i - f * h
    val B = f * g - d * i
    val C = d * h - e * g

    val det = a * A + b * B + c * C

    return DMat3.of(
            A / det,               B / det,               C / det,
            (c * h - b * i) / det, (a * i - c * g) / det, (b * g - a * h) / det,
            (b * f - c * e) / det, (c * d - a * f) / det, (a * e - b * d) / det
    )
}

fun transpose(m: DMat4) = DMat4(
        Double4(m.x.x, m.y.x, m.z.x, m.w.x),
        Double4(m.x.y, m.y.y, m.z.y, m.w.y),
        Double4(m.x.z, m.y.z, m.z.z, m.w.z),
        Double4(m.x.w, m.y.w, m.z.w, m.w.w)
)
@Suppress("UNUSED_PARAMETER") // not a priority
fun inverse(m: DMat4): DMat4 {
    TODO("Implement inverse(DMat4)") // TODO
}

fun scale(s: Double3) = DMat4(Double4(x = s.x), Double4(y = s.y), Double4(z = s.z))
fun scale(m: DMat4) = scale(m.scale)

fun translation(t: Double3) = DMat4(w = Double4(t, 1.0))
fun translation(m: DMat4) = translation(m.translation)

fun rotation(m: DMat4) = DMat4(normalize(m.right), normalize(m.up), normalize(m.forward))
fun rotation(d: Double3): DMat4 {
    val r = transform(d, ::radians)
    val c = transform(r, { x -> cos(x.toDouble()).toDouble()})
    val s = transform(r, { x -> sin(x.toDouble()).toDouble()})

    return DMat4.of(
             c.y * c.z, -c.x * s.z + s.x * s.y * c.z,  s.x * s.z + c.x * s.y * c.z, 0.0,
             c.y * s.z,  c.x * c.z + s.x * s.y * s.z, -s.x * c.z + c.x * s.y * s.z, 0.0,
            -s.y      ,  s.x * c.y                  ,  c.x * c.y                  , 0.0,
             0.0     ,  0.0                       ,  0.0                       , 1.0
    )
}
fun rotation(axis: Double3, angle: Double): DMat4 {
    val x = axis.x
    val y = axis.y
    val z = axis.z

    val r = radians(angle)
    val c = cos(r)
    val s = sin(r)
    val d = 1.0 - c

    return DMat4.of(
            x * x * d + c    , x * y * d - z * s, x * y * d + y * s, 0.0,
            y * x * d + z * s, y * y * d + c    , y * z * d - x * s, 0.0,
            z * x * d - y * s, z * y * d + x * s, z * z * d + c    , 0.0,
            0.0             , 0.0             , 0.0             , 1.0
    )
}

fun normal(m: DMat4) = scale(1.0 / Double3(length2(m.right), length2(m.up), length2(m.forward))) * m

fun lookAt(eye: Double3, target: Double3, up: Double3 = Double3(z = 1.0)): DMat4 {
    return lookTowards(eye, target - eye, up)
}

fun lookTowards(eye: Double3, forward: Double3, up: Double3 = Double3(z = 1.0)): DMat4 {
    val f = normalize(forward)
    val r = normalize(f x up)
    val u = normalize(r x f)
    return DMat4(Double4(r), Double4(u), Double4(f), Double4(eye, 1.0))
}

fun perspective(fov: Double, ratio: Double, near: Double, far: Double): DMat4 {
    val t = 1.0 / tan(radians(fov) * 0.5f)
    val a = (far + near) / (far - near)
    val b = (2.0 * far * near) / (far - near)
    val c = t / ratio
    return DMat4(Double4(x = c), Double4(y = t), Double4(z = a, w = 1.0), Double4(z = -b))
}

fun ortho(l: Double, r: Double, b: Double, t: Double, n: Double, f: Double) = DMat4(
        Double4(x = 2.0 / (r - 1.0)),
        Double4(y = 2.0 / (t - b)),
        Double4(z = -2.0 / (f - n)),
        Double4(-(r + l) / (r - l), -(t + b) / (t - b), -(f + n) / (f - n), 1.0)
)

