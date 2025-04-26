// If-else-if-else statement cyclomatic complexity test

func cycloIfElseIfElse(a: Int, b: Int, c: Int) -> Int {
    if a > b && a > c {
        return a
    } else if b > c {
        return b
    } else {
        return c
    }
}