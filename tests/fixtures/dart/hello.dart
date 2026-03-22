import 'dart:math';
import 'dart:async';

class Calculator {
  int add(int a, int b) => a + b;
  
  void print(String message) {
    print(message);
  }
}

enum Color {
  red,
  green,
  blue
}

mixin Printable {
  void printMessage();
}
