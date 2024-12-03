func f0() int {
	x := foo - bar + 1
	return x
}

func f1() {
	if foo != 0 && bar != 0 && baz != 0 && qux != 0 && quux != 0 {
		return
	}
}
