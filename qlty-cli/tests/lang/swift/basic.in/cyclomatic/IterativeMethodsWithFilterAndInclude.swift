// Filter and map method cyclomatic complexity test

func iterativeMethodsWithFilterAndMap() {
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    
    let result = numbers
        .filter { number in
            return number % 2 == 0 // Only even numbers
        }
        .map { number in
            return number * number // Square each number
        }
    
    print(result) // [4, 16, 36, 64, 100]
}