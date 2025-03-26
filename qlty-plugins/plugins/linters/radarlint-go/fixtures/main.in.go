package main

import (
	"fmt"
)

// TODO: Refactor this function to improve efficiency  // 🚨 Triggers go:S1134
func computeSum(numbers []int) int {
	sum := 0
	for _, num := range numbers {
		sum += num
	}
	return sum
}

func main() {
	nums := []int{1, 2, 3, 4, 5}
	fmt.Println("Sum:", computeSum(nums))

	// FIXME: Handle edge cases properly  // 🚨 Triggers go:S1134
}
