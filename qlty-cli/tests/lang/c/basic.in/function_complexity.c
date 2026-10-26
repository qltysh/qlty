#include <stdio.h>

void simple(void) {
}

const char* complex_func(int bar) {
    if (bar > 0) {
        if (bar > 10) {
            if (bar < 20) {
                return "teens";
            } else if (bar < 30) {
                return "twenties";
            } else if (bar < 40) {
                return "thirties";
            }
        } else if (bar > 5) {
            switch (bar) {
                case 6: return "six";
                case 7: return "seven";
                case 8: return "eight";
                case 9: return "nine";
                case 10: return "ten";
                default: return "other";
            }
        }
    } else if (bar < -100) {
        return "very negative";
    } else if (bar < -50) {
        return "negative";
    } else if (bar < -10) {
        for (int i = bar; i < 0; i++) {
            if (i % 2 == 0) {
                printf("%d\n", i);
            } else if (i % 3 == 0) {
                printf("div3\n");
            }
        }
        return "slightly negative";
    } else {
        while (bar < 0) {
            bar++;
            if (bar == -5) {
                break;
            }
        }
        return "near zero";
    }
    return "unknown";
}
