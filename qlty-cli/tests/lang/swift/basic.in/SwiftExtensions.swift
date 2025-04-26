// Swift extensions

// Original class
class Person {
    var name: String
    var age: Int
    
    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }
    
    func greet() -> String {
        return "Hello, my name is \(name)"
    }
}

// Extension to add functionality to the Person class
extension Person {
    // Add a new method
    func celebrateBirthday() {
        age += 1
        print("\(name) is now \(age) years old!")
    }
    
    // Add a computed property
    var isAdult: Bool {
        return age >= 18
    }
}

// Extension to add protocol conformance
extension Person: CustomStringConvertible {
    var description: String {
        return "\(name), \(age) years old"
    }
}