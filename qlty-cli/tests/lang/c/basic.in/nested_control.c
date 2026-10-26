#include <stdio.h>
#include <string.h>

void not_nested(const char *foo, const char *bar) {
    if ((strcmp(foo, "cat") == 0 && strcmp(bar, "dog") == 0) ||
        (strcmp(foo, "dog") == 0 && strcmp(bar, "cat") == 0)) {
        printf("Got a cat and a dog!\n");
    } else {
        printf("Got nothing\n");
    }
}

void deeply_nested(int a, int b, int c, int d) {
    if (a) {
        for (int i = 0; i < 10; i++) {
            if (b) {
                if (c) {
                    for (int j = 0; j < 3; j++) {
                        if (d) {
                            printf("deep: %d\n", j);
                        }
                    }
                }
            }
        }
    }
}
