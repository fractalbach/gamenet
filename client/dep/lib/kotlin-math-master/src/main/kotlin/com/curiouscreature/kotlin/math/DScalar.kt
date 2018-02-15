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

const val PI          = 3.141592653589793238462643383279502884197169399
const val HALF_PI     = PI * 0.5
const val TWO_PI      = PI * 2.0
const val FOUR_PI     = PI * 4.0
const val INV_PI      = 1.0f / PI
const val INV_TWO_PI  = INV_PI * 0.5
const val INV_FOUR_PI = INV_PI * 0.25

inline fun clamp(x: Double, min: Double, max: Double): Double {
    return if (x < min) min else (if (x > max) max else x)
}

inline fun mix(a: Double, b: Double, x: Double) = a * (1.0f - x) + b * x

inline fun degrees(v: Double) = v * (180.0f * INV_PI)

inline fun radians(v: Double) = v * (PI / 180.0f)

inline fun fract(v: Double) = v % 1

inline fun sqr(v: Double) = v * v
