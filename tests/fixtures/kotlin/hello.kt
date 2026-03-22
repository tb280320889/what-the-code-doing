package com.example

import java.util.List
import java.util.ArrayList

class Calculator {
    fun add(a: Int, b: Int): Int {
        return a + b
    }

    fun print(message: String) {
        println(message)
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

data class Point(val x: Double, val y: Double)

object Constants {
    const val MAX_SIZE = 100
}
