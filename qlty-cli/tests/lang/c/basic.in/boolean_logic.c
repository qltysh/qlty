#include <stdio.h>

struct BooleanLogic {
    int foo;
    int bar;
    int baz;
    int qux;
    int zoo;
    int woo;
};

void f0(void) {
    int x = 1 - 2 + 3;
}

void f1(struct BooleanLogic *b) {
    if (b->foo && b->bar && b->baz && b->qux && b->zoo && b->woo) {
        return;
    }
}

void f2(struct BooleanLogic *b) {
    if (b->foo || b->bar || b->baz || b->qux || b->zoo || b->woo) {
        printf("check passed\n");
    }
}
