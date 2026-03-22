class Calculator {
  int value = 0;

  int add(int a, int b) {
    value = a + b;
    return value;
  }

  void print(String message) {
    print(message);
  }
}

class Point {
  double x;
  double y;

  Point(this.x, this.y);
}

interface IProcessor {
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
}
