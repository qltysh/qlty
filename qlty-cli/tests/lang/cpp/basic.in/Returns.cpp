#include <string>

class Returns {
public:
    void f0() {
    }

    void f1() {
        return;
    }

    std::string f2(int x) {
        if (x == 1) {
            return "one";
        } else if (x == 2) {
            return "two";
        } else if (x == 3) {
            return "three";
        } else if (x == 4) {
            return "four";
        } else if (x == 5) {
            return "five";
        } else if (x == 6) {
            return "six";
        } else if (x == 7) {
            return "seven";
        }
        return "other";
    }
};
