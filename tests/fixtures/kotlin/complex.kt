package com.example

import java.util.List
import java.util.ArrayList
import java.util.stream.Collectors
import java.util.function.Function

class Calculator {
    private var value: Int = 0

    fun add(a: Int, b: Int): Int {
        value = a + b
        return value
    }

    fun print(message: String) {
        println(message)
    }
}

data class Point(val x: Double, val y: Double) {
    fun distanceTo(other: Point): Double {
        val dx = x - other.x
        val dy = y - other.y
        return Math.sqrt(dx * dx + dy * dy)
    }
}

interface IProcessor {
    fun process(data: String)
}

enum class Color {
    RED,
    GREEN,
    BLUE
}

object Constants {
    const val MAX_SIZE = 100
}

annotation class MyAnnotation(val value: String)

fun main() {
    val calc = Calculator()
    val result = calc.add(5, 10)
    println(result)
}
