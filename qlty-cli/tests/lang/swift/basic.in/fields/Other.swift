// Other field types

class Other {
    // Arrays
    var stringArray: [String] = ["one", "two", "three"]
    
    // Dictionaries
    var dictionary: [String: Int] = ["one": 1, "two": 2, "three": 3]
    
    // Optionals
    var optionalString: String?
    
    // Tuples
    var tuple: (String, Int) = ("test", 123)
    
    // Closures
    var closure: (Int) -> Int = { number in
        return number * 2
    }
}