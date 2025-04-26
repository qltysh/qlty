// This file is identical to another file to test duplication detection

func identicalFunction1() {
    let x = 1
    let y = 2
    let z = 3
    
    if x > y {
        print("x is greater than y")
    } else if x < y {
        print("x is less than y")
    } else {
        print("x equals y")
    }
    
    for i in 0..<10 {
        print("Counter: \(i)")
    }
}

func identicalFunction2() {
    let x = 1
    let y = 2
    let z = 3
    
    if x > y {
        print("x is greater than y")
    } else if x < y {
        print("x is less than y")
    } else {
        print("x equals y")
    }
    
    for i in 0..<10 {
        print("Counter: \(i)")
    }
}