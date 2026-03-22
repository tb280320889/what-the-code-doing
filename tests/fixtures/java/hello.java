public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }

    public void print(String message) {
        System.out.println(message);
    }
}

public class Point {
    public double x;
    public double y;
}

public interface IProcessor {
    void process(String data);
}

public enum Color {
    RED,
    GREEN,
    BLUE
}
