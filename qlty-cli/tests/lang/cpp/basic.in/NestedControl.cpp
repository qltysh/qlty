#include <iostream>
#include <vector>
#include <stdexcept>

class NestedControl {
public:
    static void notNested(const std::string& foo, const std::string& bar) {
        if ((foo == "cat" && bar == "dog") || (foo == "dog" && bar == "cat")) {
            std::cout << "Got a cat and a dog!" << std::endl;
        } else {
            std::cout << "Got nothing" << std::endl;
        }
    }

    static void deeplyNested(bool a, bool b, bool c, bool d, bool e, bool f) {
        if (a) {
            for (int i = 0; i < 10; i++) {
                if (b) {
                    try {
                        if (c) {
                            for (auto& val : std::vector<int>{1, 2, 3}) {
                                if (d) {
                                    std::cout << "deep: " << val << std::endl;
                                }
                            }
                        }
                    } catch (const std::exception& ex) {
                        std::cout << "error" << std::endl;
                    }
                }
            }
        }
    }
};
