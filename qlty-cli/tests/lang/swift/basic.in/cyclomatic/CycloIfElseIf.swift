// If-else-if statement cyclomatic complexity test

func cycloIfElseIf(a: Int, b: Int, c: Int) -> Int {
    if a > b && a > c {
        return a
    } else if b > c {
        return b
    }
    return c
}