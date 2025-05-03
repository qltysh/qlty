// Testing multiple conditionals for cognitive complexity

func multipleConditions(a: Int, b: Int, c: Int) -> String {
    if a > b {
        if b > c {
            return "a > b > c"
        } else {
            return "a > b, b <= c"
        }
    } else if a == b {
        if b > c {
            return "a = b > c"
        } else if b == c {
            return "a = b = c"
        } else {
            return "a = b < c"
        }
    } else {
        if a > c {
            return "b > a > c"
        } else if a == c {
            return "b > a = c"
        } else {
            return "b > a, a < c"
        }
    }
}