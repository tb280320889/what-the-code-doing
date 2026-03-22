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
}
