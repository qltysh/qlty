#include <stdlib.h>

double compute_stats1(double *numbers, int count) {
    double sum = 0;
    for (int i = 0; i < count; i++) {
        sum += numbers[i];
    }
    double mean = sum / count;

    double *sorted = malloc(count * sizeof(double));
    for (int i = 0; i < count; i++) {
        sorted[i] = numbers[i];
    }

    double median;
    if (count % 2 == 0) {
        median = (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0;
    } else {
        median = sorted[count / 2];
    }

    free(sorted);
    return mean + median;
}

double compute_stats2(double *numbers, int count) {
    double sum = 0;
    for (int i = 0; i < count; i++) {
        sum += numbers[i];
    }
    double mean = sum / count;

    double *sorted = malloc(count * sizeof(double));
    for (int i = 0; i < count; i++) {
        sorted[i] = numbers[i];
    }

    double median;
    if (count % 2 == 0) {
        median = (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0;
    } else {
        median = sorted[count / 2];
    }

    free(sorted);
    return mean + median;
}
