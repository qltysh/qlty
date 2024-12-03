func f0() {
}

func f1(dog string, cat string) {
}

func f2(a int, b int, c int, d int, e int, f int) {
}

func f3() {
	// Function calls don't count, only function definitions
	foo := bar(1, 2, 3, 4, 5)
	fmt.Println(foo)
}
