package material

fun uValue(value: dynamic): dynamic {
    val o: dynamic = object{}
    o["value"] = value
    return o
}
