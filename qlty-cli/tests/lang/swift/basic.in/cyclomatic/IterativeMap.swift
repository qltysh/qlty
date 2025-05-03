// Map iteration cyclomatic complexity test

func iterativeMap() {
    let numbers = [1, 2, 3, 4, 5]
    
    let doubled = numbers.map { number in
        return number * 2
    }
    
    print(doubled)
}