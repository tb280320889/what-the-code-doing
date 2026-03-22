package com.example;

import java.util.List;
import java.util.ArrayList;
import java.util.stream.Collectors;
import java.util.function.Function;

public class Calculator {
    private int value;

    public Calculator() {
        this.value = 0;
    }

    public int add(int a, int b) {
        this.value = a + b;
        return this.value;
    }

    public void print(String message) {
        System.out.println(message);
    }
}

public class Point {
    public double x;
    public double y;

    public Point(double x, double y) {
        this.x = x;
        this.y = y;
    }

    public double distanceTo(Point other) {
        double dx = this.x - other.x;
        double dy = this.y - other.y;
        return Math.sqrt(dx * dx + dy * dy);
    }
}

public interface IProcessor {
    void process(String data);
}

public enum Color {
    RED,
    GREEN,
    BLUE
}

public @interface MyAnnotation {
    String value();
}

public record Person(String name, int age) {
    public String greet() {
        return "Hello, " + name;
    }
}

class Main {
    public static void main(String[] args) {
        Calculator calc = new Calculator();
        int result = calc.add(5, 10);
        System.out.println(result);
    }
}
