// Swift closures and functional programming

// Simple closure
let simpleClosure = { (name: String) -> String in
    return "Hello, \(name)!"
}

// Higher-order function that takes a closure
func performOperation(on numbers: [Int], using operation: (Int) -> Int) -> [Int] {
    var result = [Int]()
    for number in numbers {
        result.append(operation(number))
    }
    return result
}

// Function that returns a closure
func makeAdder(amount: Int) -> (Int) -> Int {
    return { number in
        return number + amount
    }
}

// Using functional programming patterns
func functionalProgrammingExamples() {
    let numbers = [1, 2, 3, 4, 5]
    
    // Map
    let doubled = numbers.map { $0 * 2 }
    
    // Filter
    let evens = numbers.filter { $0 % 2 == 0 }
    
    // Reduce
    let sum = numbers.reduce(0) { $0 + $1 }
    
    // Chaining operations
    let sumOfSquaresOfEvens = numbers
        .filter { $0 % 2 == 0 }
        .map { $0 * $0 }
        .reduce(0) { $0 + $1 }
    
    print("Doubled: \(doubled)")
    print("Evens: \(evens)")
    print("Sum: \(sum)")
    print("Sum of squares of evens: \(sumOfSquaresOfEvens)")
}