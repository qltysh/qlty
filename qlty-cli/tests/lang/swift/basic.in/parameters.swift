func f0() {
    print("No parameters")
}

func f1(dog: String, cat: String) {
    print("Two parameters: \(dog), \(cat)")
}

func f2(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int) {
    print("Six parameters")
}

func f3() {
    // Function calls don't count, only function definitions
    let result = bar(param1: 1, param2: 2, param3: 3, param4: 4)
}

func bar(param1: Int, param2: Int, param3: Int, param4: Int) -> Int {
    return param1 + param2 + param3 + param4
}