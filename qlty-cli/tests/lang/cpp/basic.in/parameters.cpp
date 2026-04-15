#include <string>

class Parameters {
public:
    static void f0() {
    }

    static void f1(int dog, int cat) {
    }

    static void f2(int a, int b, int c, int d, int e, int f) {
    }

    static void f3() {
        int foo = bar(1, 2, 3, 4);
    }

    static int bar(int a, int b, int c, int d) {
        return a + b + c + d;
    }
};
