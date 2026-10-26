#include <iostream>
#include <vector>
#include <algorithm>

using namespace std;
using MyInt = int;

double computeStats1(vector<double>& numbers) {
    double sum = 0;
    for (auto& num : numbers) {
        sum += num;
    }
    double mean = sum / numbers.size();

    vector<double> sorted(numbers);
    sort(sorted.begin(), sorted.end());

    double median;
    MyInt length = sorted.size();
    if (length % 2 == 0) {
        median = (sorted[length / 2 - 1] + sorted[length / 2]) / 2.0;
    } else {
        median = sorted[length / 2];
    }

    return mean + median;
}

double computeStats2(vector<double>& numbers) {
    double sum = 0;
    for (auto& num : numbers) {
        sum += num;
    }
    double mean = sum / numbers.size();

    vector<double> sorted(numbers);
    sort(sorted.begin(), sorted.end());

    double median;
    MyInt length = sorted.size();
    if (length % 2 == 0) {
        median = (sorted[length / 2 - 1] + sorted[length / 2]) / 2.0;
    } else {
        median = sorted[length / 2];
    }

    return mean + median;
}
