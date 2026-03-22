package com.example;

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
