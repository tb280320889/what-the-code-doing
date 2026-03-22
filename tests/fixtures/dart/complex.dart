import 'dart:math';
import 'dart:async';
import 'package:flutter/material.dart';

class Calculator {
  int _value = 0;

  Calculator();

  int add(int a, int b) {
    _value = a + b;
    return _value;
  }

  void print(String message) {
    print(message);
  }
}

class Point {
  double x;
  double y;

  Point(this.x, this.y);

  double distanceTo(Point other) {
    return sqrt(pow(x - other.x, 2) + pow(y - other.y, 2));
  }
}

abstract class IProcessor {
  void process(String data);
}

enum Color {
  red,
  green,
  blue
}

mixin Printable {
  void printMessage();
}

extension StringExtensions on String {
  String trimmed() {
    return trim();
  }

  bool get isNullOrEmpty => isEmpty;
}

void main() {
  var calc = Calculator();
  int result = calc.add(5, 10);
  print(result);
}
