#include <stdio.h>
#include <stdlib.h>

#define MAX_SIZE 100

struct Point {
    int x;
    int y;
};

enum Color {
    RED,
    GREEN,
    BLUE
};

typedef struct Point Point;

int add(int a, int b) {
    return a + b;
}

void process(const char* data, size_t len) {
    for (size_t i = 0; i < len; i++) {
        printf("%c", data[i]);
    }
}

int main() {
    Point p = {10, 20};
    int result = add(p.x, p.y);
    return 0;
}
