package com.example

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

data class Point(val x: Double, val y: Double)

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
