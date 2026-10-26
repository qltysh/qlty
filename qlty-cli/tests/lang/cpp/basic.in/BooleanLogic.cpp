#include <iostream>

class BooleanLogic {
public:
    bool foo;
    bool bar;
    bool baz;
    bool qux;
    bool zoo;
    bool woo;

    void f0() {
        int x = 1 - 2 + 3;
    }

    void f1() {
        if (foo && bar && baz && qux && zoo && woo) {
            return;
        }
    }

    void f2() {
        auto check = [this]() {
            return foo || bar || baz || qux || zoo || woo;
        };
        if (check()) {
            std::cout << "lambda check passed" << std::endl;
        }
    }
};
