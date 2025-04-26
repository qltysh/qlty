// If statement with boolean operators cyclomatic complexity test

func ifWithBool(a: Bool, b: Bool, c: Bool) -> Bool {
    if a && b || c {
        return true
    }
    return false
}