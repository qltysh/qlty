func f1() {
    let bar = true
    let baz = true
    let qux = true
    let quux = true
    
    if bar {
        if baz {
            if qux {
                if quux {
                    print("Deeply nested!")
                }
            }
        }
    }
}

func f2(foo: Int) -> String {
    if foo == 1 {
        return "bar1"
    } else if foo == 2 {
        return "bar2"
    } else if foo == 3 {
        return "bar3"
    } else if foo == 4 {
        return "bar4"
    } else if foo == 5 {
        return "bar5"
    } else if foo == 6 {
        return "bar6"
    } else if foo == 7 {
        return "bar7"
    } else if foo == 8 {
        return "bar8"
    } else if foo == 9 {
        return "bar9"
    } else if foo == 10 {
        return "bar10"
    } else {
        fatalError("Invalid foo value")
    }
}