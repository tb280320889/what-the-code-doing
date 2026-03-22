#include <iostream>
#include <vector>
#include <string>
#include <algorithm>

#define MAX_SIZE 100

namespace myapp {
    class Calculator {
    public:
        int add(int a, int b);
        void print(const std::string& msg);
    private:
        int value;
    };

    struct Point {
        double x;
        double y;
    };

    enum Color {
        RED,
        GREEN,
        BLUE
    };

    template<typename T>
    T max(T a, T b) {
        return (a > b) ? a : b;
    }
}

int main() {
    myapp::Calculator calc;
    int result = calc.add(5, 10);
    return 0;
}
