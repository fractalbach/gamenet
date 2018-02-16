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

@file:Suppress("NOTHING_TO_INLINE")

package com.curiouscreature.kotlin.math

import info.laht.threekt.math.Vector2
import info.laht.threekt.math.Vector3
import info.laht.threekt.math.Vector4
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

enum class DVectorComponent {
    X, Y, Z, W,
    R, G, B, A,
    S, T, P, Q
}

data class Double2(var x: Double = 0.0, var y: Double = 0.0) {
    constructor(v: Double2) : this(v.x, v.y)
    constructor(v: Vector2) : this(v.x, v.y)

    inline var r: Double
        get() = x
        set(value) {
            x = value
        }
    inline var g: Double
        get() = y
        set(value) {
            y = value
        }

    inline var s: Double
        get() = x
        set(value) {
            x = value
        }
    inline var t: Double
        get() = y
        set(value) {
            y = value
        }

    inline var xy: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }
    inline var rg: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }
    inline var st: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }

    operator fun get(index: VectorComponent) = when (index) {
        VectorComponent.X, VectorComponent.R, VectorComponent.S -> x
        VectorComponent.Y, VectorComponent.G, VectorComponent.T -> y
        else -> throw IllegalArgumentException("index must be X, Y, R, G, S or T")
    }

    operator fun get(index1: VectorComponent, index2: VectorComponent): Double2 {
        return Double2(get(index1), get(index2))
    }

    operator fun get(index: Int) = when (index) {
        0 -> x
        1 -> y
        else -> throw IllegalArgumentException("index must be in 0..1")
    }

    operator fun get(index1: Int, index2: Int) = Double2(get(index1), get(index2))

    inline operator fun invoke(index: Int) = get(index - 1)

    operator fun set(index: Int, v: Double) = when (index) {
        0 -> x = v
        1 -> y = v
        else -> throw IllegalArgumentException("index must be in 0..1")
    }

    operator fun set(index1: Int, index2: Int, v: Double) {
        set(index1, v)
        set(index2, v)
    }

    operator fun set(index: VectorComponent, v: Double) = when (index) {
        VectorComponent.X, VectorComponent.R, VectorComponent.S -> x = v
        VectorComponent.Y, VectorComponent.G, VectorComponent.T -> y = v
        else -> throw IllegalArgumentException("index must be X, Y, R, G, S or T")
    }

    operator fun set(index1: VectorComponent, index2: VectorComponent, v: Double) {
        set(index1, v)
        set(index2, v)
    }

    operator fun unaryMinus() = Double2(-x, -y)
    operator fun inc(): Double2 {
        x += 1.0
        y += 1.0
        return this
    }

    operator fun dec(): Double2 {
        x -= 1.0
        y -= 1.0
        return this
    }

    inline operator fun plus(v: Double) = Double2(x + v, y + v)
    inline operator fun minus(v: Double) = Double2(x - v, y - v)
    inline operator fun times(v: Double) = Double2(x * v, y * v)
    inline operator fun div(v: Double) = Double2(x / v, y / v)

    inline operator fun plus(v: Double2) = Double2(x + v.x, y + v.y)
    inline operator fun minus(v: Double2) = Double2(x - v.x, y - v.y)
    inline operator fun times(v: Double2) = Double2(x * v.x, y * v.y)
    inline operator fun div(v: Double2) = Double2(x / v.x, y / v.y)

    inline fun transform(block: (Double) -> Double): Double2 {
        x = block(x)
        y = block(y)
        return this
    }
}

data class Double3(var x: Double = 0.0, var y: Double = 0.0, var z: Double = 0.0) {
    constructor(v: Double2, z: Double = 0.0) : this(v.x, v.y, z)
    constructor(v: Double3) : this(v.x, v.y, v.z)
    constructor(v: Vector3) : this(v.x, v.y, v.z)

    inline var r: Double
        get() = x
        set(value) {
            x = value
        }
    inline var g: Double
        get() = y
        set(value) {
            y = value
        }
    inline var b: Double
        get() = z
        set(value) {
            z = value
        }

    inline var s: Double
        get() = x
        set(value) {
            x = value
        }
    inline var t: Double
        get() = y
        set(value) {
            y = value
        }
    inline var p: Double
        get() = z
        set(value) {
            z = value
        }

    inline var xy: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }
    inline var rg: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }
    inline var st: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }

    inline var rgb: Double3
        get() = Double3(x, y, z)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
        }
    inline var xyz: Double3
        get() = Double3(x, y, z)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
        }
    inline var stp: Double3
        get() = Double3(x, y, z)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
        }

    operator fun get(index: VectorComponent) = when (index) {
        VectorComponent.X, VectorComponent.R, VectorComponent.S -> x
        VectorComponent.Y, VectorComponent.G, VectorComponent.T -> y
        VectorComponent.Z, VectorComponent.B, VectorComponent.P -> z
        else -> throw IllegalArgumentException("index must be X, Y, Z, R, G, B, S, T or P")
    }

    operator fun get(index1: VectorComponent, index2: VectorComponent): Double2 {
        return Double2(get(index1), get(index2))
    }
    operator fun get(
            index1: VectorComponent, index2: VectorComponent, index3: VectorComponent): Double3 {
        return Double3(get(index1), get(index2), get(index3))
    }

    operator fun get(index: Int) = when (index) {
        0 -> x
        1 -> y
        2 -> z
        else -> throw IllegalArgumentException("index must be in 0..2")
    }

    operator fun get(index1: Int, index2: Int) = Double2(get(index1), get(index2))
    operator fun get(index1: Int, index2: Int, index3: Int): Double3 {
        return Double3(get(index1), get(index2), get(index3))
    }

    inline operator fun invoke(index: Int) = get(index - 1)

    operator fun set(index: Int, v: Double) = when (index) {
        0 -> x = v
        1 -> y = v
        2 -> z = v
        else -> throw IllegalArgumentException("index must be in 0..2")
    }

    operator fun set(index1: Int, index2: Int, v: Double) {
        set(index1, v)
        set(index2, v)
    }

    operator fun set(index1: Int, index2: Int, index3: Int, v: Double) {
        set(index1, v)
        set(index2, v)
        set(index3, v)
    }

    operator fun set(index: VectorComponent, v: Double) = when (index) {
        VectorComponent.X, VectorComponent.R, VectorComponent.S -> x = v
        VectorComponent.Y, VectorComponent.G, VectorComponent.T -> y = v
        VectorComponent.Z, VectorComponent.B, VectorComponent.P -> z = v
        else -> throw IllegalArgumentException("index must be X, Y, Z, R, G, B, S, T or P")
    }

    operator fun set(index1: VectorComponent, index2: VectorComponent, v: Double) {
        set(index1, v)
        set(index2, v)
    }

    operator fun set(
            index1: VectorComponent,
            index2: VectorComponent,
            index3: VectorComponent,
            v: Double) {
        set(index1, v)
        set(index2, v)
        set(index3, v)
    }

    operator fun unaryMinus() = Double3(-x, -y, -z)
    operator fun inc(): Double3 {
        x += 1.0
        y += 1.0
        z += 1.0
        return this
    }

    operator fun dec(): Double3 {
        x -= 1.0
        y -= 1.0
        z -= 1.0
        return this
    }

    inline operator fun plus(v: Double) = Double3(x + v, y + v, z + v)
    inline operator fun minus(v: Double) = Double3(x - v, y - v, z - v)
    inline operator fun times(v: Double) = Double3(x * v, y * v, z * v)
    inline operator fun div(v: Double) = Double3(x / v, y / v, z / v)

    inline operator fun plus(v: Double2) = Double3(x + v.x, y + v.y, z)
    inline operator fun minus(v: Double2) = Double3(x - v.x, y - v.y, z)
    inline operator fun times(v: Double2) = Double3(x * v.x, y * v.y, z)
    inline operator fun div(v: Double2) = Double3(x / v.x, y / v.y, z)

    inline operator fun plus(v: Double3) = Double3(x + v.x, y + v.y, z + v.z)
    inline operator fun minus(v: Double3) = Double3(x - v.x, y - v.y, z - v.z)
    inline operator fun times(v: Double3) = Double3(x * v.x, y * v.y, z * v.z)
    inline operator fun div(v: Double3) = Double3(x / v.x, y / v.y, z / v.z)

    inline fun transform(block: (Double) -> Double): Double3 {
        x = block(x)
        y = block(y)
        z = block(z)
        return this
    }
}

data class Double4(
        var x: Double = 0.0,
        var y: Double = 0.0,
        var z: Double = 0.0,
        var w: Double = 0.0) {
    constructor(v: Double2, z: Double = 0.0, w: Double = 0.0) : this(v.x, v.y, z, w)
    constructor(v: Double3, w: Double = 0.0) : this(v.x, v.y, v.z, w)
    constructor(v: Double4) : this(v.x, v.y, v.z, v.w)
    constructor(v: Vector4) : this(v.x, v.y, v.z, v.w)

    inline var r: Double
        get() = x
        set(value) {
            x = value
        }
    inline var g: Double
        get() = y
        set(value) {
            y = value
        }
    inline var b: Double
        get() = z
        set(value) {
            z = value
        }
    inline var a: Double
        get() = w
        set(value) {
            w = value
        }

    inline var s: Double
        get() = x
        set(value) {
            x = value
        }
    inline var t: Double
        get() = y
        set(value) {
            y = value
        }
    inline var p: Double
        get() = z
        set(value) {
            z = value
        }
    inline var q: Double
        get() = w
        set(value) {
            w = value
        }

    inline var xy: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }
    inline var rg: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }
    inline var st: Double2
        get() = Double2(x, y)
        set(value) {
            x = value.x
            y = value.y
        }

    inline var rgb: Double3
        get() = Double3(x, y, z)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
        }
    inline var xyz: Double3
        get() = Double3(x, y, z)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
        }
    inline var stp: Double3
        get() = Double3(x, y, z)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
        }

    inline var rgba: Double4
        get() = Double4(x, y, z, w)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
            w = value.w
        }
    inline var xyzw: Double4
        get() = Double4(x, y, z, w)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
            w = value.w
        }
    inline var stpq: Double4
        get() = Double4(x, y, z, w)
        set(value) {
            x = value.x
            y = value.y
            z = value.z
            w = value.w
        }

    operator fun get(index: VectorComponent) = when (index) {
        VectorComponent.X, VectorComponent.R, VectorComponent.S -> x
        VectorComponent.Y, VectorComponent.G, VectorComponent.T -> y
        VectorComponent.Z, VectorComponent.B, VectorComponent.P -> z
        VectorComponent.W, VectorComponent.A, VectorComponent.Q -> w
    }

    operator fun get(index1: VectorComponent, index2: VectorComponent): Double2 {
        return Double2(get(index1), get(index2))
    }
    operator fun get(
            index1: VectorComponent,
            index2: VectorComponent,
            index3: VectorComponent): Double3 {
        return Double3(get(index1), get(index2), get(index3))
    }
    operator fun get(
            index1: VectorComponent,
            index2: VectorComponent,
            index3: VectorComponent,
            index4: VectorComponent): Double4 {
        return Double4(get(index1), get(index2), get(index3), get(index4))
    }

    operator fun get(index: Int) = when (index) {
        0 -> x
        1 -> y
        2 -> z
        3 -> w
        else -> throw IllegalArgumentException("index must be in 0..3")
    }

    operator fun get(index1: Int, index2: Int) = Double2(get(index1), get(index2))
    operator fun get(index1: Int, index2: Int, index3: Int): Double3 {
        return Double3(get(index1), get(index2), get(index3))
    }
    operator fun get(index1: Int, index2: Int, index3: Int, index4: Int): Double4 {
        return Double4(get(index1), get(index2), get(index3), get(index4))
    }

    inline operator fun invoke(index: Int) = get(index - 1)

    operator fun set(index: Int, v: Double) = when (index) {
        0 -> x = v
        1 -> y = v
        2 -> z = v
        3 -> w = v
        else -> throw IllegalArgumentException("index must be in 0..3")
    }

    operator fun set(index1: Int, index2: Int, v: Double) {
        set(index1, v)
        set(index2, v)
    }

    operator fun set(index1: Int, index2: Int, index3: Int, v: Double) {
        set(index1, v)
        set(index2, v)
        set(index3, v)
    }

    operator fun set(index1: Int, index2: Int, index3: Int, index4: Int, v: Double) {
        set(index1, v)
        set(index2, v)
        set(index3, v)
        set(index4, v)
    }

    operator fun set(index: VectorComponent, v: Double) = when (index) {
        VectorComponent.X, VectorComponent.R, VectorComponent.S -> x = v
        VectorComponent.Y, VectorComponent.G, VectorComponent.T -> y = v
        VectorComponent.Z, VectorComponent.B, VectorComponent.P -> z = v
        VectorComponent.W, VectorComponent.A, VectorComponent.Q -> w = v
    }

    operator fun set(index1: VectorComponent, index2: VectorComponent, v: Double) {
        set(index1, v)
        set(index2, v)
    }

    operator fun set(index1: VectorComponent, index2: VectorComponent, index3: VectorComponent,
            v: Double) {
        set(index1, v)
        set(index2, v)
        set(index3, v)
    }

    operator fun set(index1: VectorComponent, index2: VectorComponent,
            index3: VectorComponent, index4: VectorComponent, v: Double) {
        set(index1, v)
        set(index2, v)
        set(index3, v)
        set(index4, v)
    }

    operator fun unaryMinus() = Double4(-x, -y, -z, -w)
    operator fun inc(): Double4 {
        x += 1.0
        y += 1.0
        z += 1.0
        w += 1.0
        return this
    }

    operator fun dec(): Double4 {
        x -= 1.0
        y -= 1.0
        z -= 1.0
        w -= 1.0
        return this
    }

    inline operator fun plus(v: Double) = Double4(x + v, y + v, z + v, w + v)
    inline operator fun minus(v: Double) = Double4(x - v, y - v, z - v, w - v)
    inline operator fun times(v: Double) = Double4(x * v, y * v, z * v, w * v)
    inline operator fun div(v: Double) = Double4(x / v, y / v, z / v, z / v)

    inline operator fun plus(v: Double2) = Double4(x + v.x, y + v.y, z, w)
    inline operator fun minus(v: Double2) = Double4(x - v.x, y - v.y, z, w)
    inline operator fun times(v: Double2) = Double4(x * v.x, y * v.y, z, w)
    inline operator fun div(v: Double2) = Double4(x / v.x, y / v.y, z, w)

    inline operator fun plus(v: Double3) = Double4(x + v.x, y + v.y, z + v.z, w)
    inline operator fun minus(v: Double3) = Double4(x - v.x, y - v.y, z - v.z, w)
    inline operator fun times(v: Double3) = Double4(x * v.x, y * v.y, z * v.z, w)
    inline operator fun div(v: Double3) = Double4(x / v.x, y / v.y, z / v.z, w)

    inline operator fun plus(v: Double4) = Double4(x + v.x, y + v.y, z + v.z, w + v.w)
    inline operator fun minus(v: Double4) = Double4(x - v.x, y - v.y, z - v.z, w - v.w)
    inline operator fun times(v: Double4) = Double4(x * v.x, y * v.y, z * v.z, w * v.w)
    inline operator fun div(v: Double4) = Double4(x / v.x, y / v.y, z / v.z, w / v.w)

    inline fun transform(block: (Double) -> Double): Double4 {
        x = block(x)
        y = block(y)
        z = block(z)
        w = block(w)
        return this
    }
}

inline operator fun Double.plus(v: Double2) = Double2(this + v.x, this + v.y)
inline operator fun Double.minus(v: Double2) = Double2(this - v.x, this - v.y)
inline operator fun Double.times(v: Double2) = Double2(this * v.x, this * v.y)
inline operator fun Double.div(v: Double2) = Double2(this / v.x, this / v.y)

inline fun abs(v: Double2) = Double2(abs(v.x), abs(v.y))
inline fun length(v: Double2) = sqrt(v.x * v.x + v.y * v.y)
inline fun length2(v: Double2) = v.x * v.x + v.y * v.y
inline fun distance(a: Double2, b: Double2) = length(a - b)
inline fun dot(a: Double2, b: Double2) = a.x * b.x + a.y * b.y
fun normalize(v: Double2): Double2 {
    val l = 1.0 / length(v)
    return Double2(v.x * l, v.y * l)
}

inline fun reflect(i: Double2, n: Double2) = i - 2.0 * dot(n, i) * n
fun refract(i: Double2, n: Double2, eta: Double): Double2 {
    val d = dot(n, i)
    val k = 1.0 - eta * eta * (1.0 - sqr(d))
    return if (k < 0.0) Double2(0.0) else eta * i - (eta * d + sqrt(k)) * n
}

inline fun clamp(v: Double2, min: Double, max: Double): Double2 {
    return Double2(
            clamp(v.x, min, max),
            clamp(v.y, min, max))
}

inline fun clamp(v: Double2, min: Double2, max: Double2): Double2 {
    return Double2(
            clamp(v.x, min.x, max.x),
            clamp(v.y, min.y, max.y))
}

inline fun mix(a: Double2, b: Double2, x: Double): Double2 {
    return Double2(
            mix(a.x, b.x, x),
            mix(a.y, b.y, x))
}

inline fun mix(a: Double2, b: Double2, x: Double2): Double2 {
    return Double2(
            mix(a.x, b.x, x.x),
            mix(a.y, b.y, x.y))
}

inline fun min(v: Double2) = min(v.x, v.y)
inline fun min(a: Double2, b: Double2) = Double2(min(a.x, b.x), min(a.y, b.y))
inline fun max(v: Double2) = max(v.x, v.y)
inline fun max(a: Double2, b: Double2) = Double2(max(a.x, b.x), max(a.y, b.y))

inline fun transform(v: Double2, block: (Double) -> Double) = v.copy().transform(block)

inline fun lessThan(a: Double2, b: Double) = Bool2(a.x < b, a.y < b)
inline fun lessThan(a: Double2, b: Double2) = Bool2(a.x < b.x, a.y < b.y)
inline fun lessThanEqual(a: Double2, b: Double) = Bool2(a.x <= b, a.y <= b)
inline fun lessThanEqual(a: Double2, b: Double2) = Bool2(a.x <= b.x, a.y <= b.y)
inline fun greaterThan(a: Double2, b: Double) = Bool2(a.x > b, a.y > b)
inline fun greaterThan(a: Double2, b: Double2) = Bool2(a.x > b.y, a.y > b.y)
inline fun greaterThanEqual(a: Double2, b: Double) = Bool2(a.x >= b, a.y >= b)
inline fun greaterThanEqual(a: Double2, b: Double2) = Bool2(a.x >= b.x, a.y >= b.y)
inline fun equal(a: Double2, b: Double) = Bool2(a.x == b, a.y == b)
inline fun equal(a: Double2, b: Double2) = Bool2(a.x == b.x, a.y == b.y)
inline fun notEqual(a: Double2, b: Double) = Bool2(a.x != b, a.y != b)
inline fun notEqual(a: Double2, b: Double2) = Bool2(a.x != b.x, a.y != b.y)

inline operator fun Double.plus(v: Double3) = Double3(this + v.x, this + v.y, this + v.z)
inline operator fun Double.minus(v: Double3) = Double3(this - v.x, this - v.y, this - v.z)
inline operator fun Double.times(v: Double3) = Double3(this * v.x, this * v.y, this * v.z)
inline operator fun Double.div(v: Double3) = Double3(this / v.x, this / v.y, this / v.z)

inline fun abs(v: Double3) = Double3(abs(v.x), abs(v.y), abs(v.z))
inline fun length(v: Double3) = sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
inline fun length2(v: Double3) = v.x * v.x + v.y * v.y + v.z * v.z
inline fun distance(a: Double3, b: Double3) = length(a - b)
inline fun dot(a: Double3, b: Double3) = a.x * b.x + a.y * b.y + a.z * b.z
inline fun cross(a: Double3, b: Double3): Double3 {
    return Double3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}
inline infix fun Double3.x(v: Double3): Double3 {
    return Double3(y * v.z - z * v.y, z * v.x - x * v.z, x * v.y - y * v.x)
}
fun normalize(v: Double3): Double3 {
    val l = 1.0 / length(v)
    return Double3(v.x * l, v.y * l, v.z * l)
}

inline fun reflect(i: Double3, n: Double3) = i - 2.0 * dot(n, i) * n
fun refract(i: Double3, n: Double3, eta: Double): Double3 {
    val d = dot(n, i)
    val k = 1.0 - eta * eta * (1.0 - sqr(d))
    return if (k < 0.0) Double3(0.0) else eta * i - (eta * d + sqrt(k)) * n
}

inline fun clamp(v: Double3, min: Double, max: Double): Double3 {
    return Double3(
            clamp(v.x, min, max),
            clamp(v.y, min, max),
            clamp(v.z, min, max))
}

inline fun clamp(v: Double3, min: Double3, max: Double3): Double3 {
    return Double3(
            clamp(v.x, min.x, max.x),
            clamp(v.y, min.y, max.y),
            clamp(v.z, min.z, max.z))
}

inline fun mix(a: Double3, b: Double3, x: Double): Double3 {
    return Double3(
            mix(a.x, b.x, x),
            mix(a.y, b.y, x),
            mix(a.z, b.z, x))
}

inline fun mix(a: Double3, b: Double3, x: Double3): Double3 {
    return Double3(
            mix(a.x, b.x, x.x),
            mix(a.y, b.y, x.y),
            mix(a.z, b.z, x.z))
}

inline fun min(v: Double3) = min(v.x, min(v.y, v.z))
inline fun min(a: Double3, b: Double3) = Double3(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z))
inline fun max(v: Double3) = max(v.x, max(v.y, v.z))
inline fun max(a: Double3, b: Double3) = Double3(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z))

inline fun transform(v: Double3, block: (Double) -> Double) = v.copy().transform(block)

inline fun lessThan(a: Double3, b: Double) = Bool3(a.x < b, a.y < b, a.z < b)
inline fun lessThan(a: Double3, b: Double3) = Bool3(a.x < b.x, a.y < b.y, a.z < b.z)
inline fun lessThanEqual(a: Double3, b: Double) = Bool3(a.x <= b, a.y <= b, a.z <= b)
inline fun lessThanEqual(a: Double3, b: Double3) = Bool3(a.x <= b.x, a.y <= b.y, a.z <= b.z)
inline fun greaterThan(a: Double3, b: Double) = Bool3(a.x > b, a.y > b, a.z > b)
inline fun greaterThan(a: Double3, b: Double3) = Bool3(a.x > b.y, a.y > b.y, a.z > b.z)
inline fun greaterThanEqual(a: Double3, b: Double) = Bool3(a.x >= b, a.y >= b, a.z >= b)
inline fun greaterThanEqual(a: Double3, b: Double3) = Bool3(a.x >= b.x, a.y >= b.y, a.z >= b.z)
inline fun equal(a: Double3, b: Double) = Bool3(a.x == b, a.y == b, a.z == b)
inline fun equal(a: Double3, b: Double3) = Bool3(a.x == b.x, a.y == b.y, a.z == b.z)
inline fun notEqual(a: Double3, b: Double) = Bool3(a.x != b, a.y != b, a.z != b)
inline fun notEqual(a: Double3, b: Double3) = Bool3(a.x != b.x, a.y != b.y, a.z != b.z)

inline operator fun Double.plus(v: Double4) = Double4(this + v.x, this + v.y, this + v.z, this + v.w)
inline operator fun Double.minus(v: Double4) = Double4(this - v.x, this - v.y, this - v.z, this - v.w)
inline operator fun Double.times(v: Double4) = Double4(this * v.x, this * v.y, this * v.z, this * v.w)
inline operator fun Double.div(v: Double4) = Double4(this / v.x, this / v.y, this / v.z, this / v.w)

inline fun abs(v: Double4) = Double4(abs(v.x), abs(v.y), abs(v.z), abs(v.w))
inline fun length(v: Double4) = sqrt(v.x * v.x + v.y * v.y + v.z * v.z + v.w * v.w)
inline fun length2(v: Double4) = v.x * v.x + v.y * v.y + v.z * v.z + v.w * v.w
inline fun distance(a: Double4, b: Double4) = length(a - b)
inline fun dot(a: Double4, b: Double4) = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
fun normalize(v: Double4): Double4 {
    val l = 1.0 / length(v)
    return Double4(v.x * l, v.y * l, v.z * l, v.w * l)
}

inline fun clamp(v: Double4, min: Double, max: Double): Double4 {
    return Double4(
            clamp(v.x, min, max),
            clamp(v.y, min, max),
            clamp(v.z, min, max),
            clamp(v.w, min, max))
}

inline fun clamp(v: Double4, min: Double4, max: Double4): Double4 {
    return Double4(
            clamp(v.x, min.x, max.x),
            clamp(v.y, min.y, max.y),
            clamp(v.z, min.z, max.z),
            clamp(v.w, min.z, max.w))
}

inline fun mix(a: Double4, b: Double4, x: Double): Double4 {
    return Double4(
            mix(a.x, b.x, x),
            mix(a.y, b.y, x),
            mix(a.z, b.z, x),
            mix(a.w, b.w, x))
}

inline fun mix(a: Double4, b: Double4, x: Double4): Double4 {
    return Double4(
            mix(a.x, b.x, x.x),
            mix(a.y, b.y, x.y),
            mix(a.z, b.z, x.z),
            mix(a.w, b.w, x.w))
}

inline fun min(v: Double4) = min(v.x, min(v.y, min(v.z, v.w)))
inline fun min(a: Double4, b: Double4): Double4 {
    return Double4(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z), min(a.w, b.w))
}
inline fun max(v: Double4) = max(v.x, max(v.y, max(v.z, v.w)))
inline fun max(a: Double4, b: Double4): Double4 {
    return Double4(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z), max(a.w, b.w))
}

inline fun transform(v: Double4, block: (Double) -> Double) = v.copy().transform(block)
