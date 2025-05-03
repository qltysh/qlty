class Person {
    var name: String
    var age: Int
    
    init(name: String, age: Int) {
        self.name = name
        self.age = age
    }
    
    func greet() -> String {
        return "Hello, my name is \(name) and I am \(age) years old."
    }
    
    func haveBirthday() {
        self.age += 1
        print("\(name) is now \(age) years old!")
    }
}

// Using the class
let john = Person(name: "John", age: 30)
john.greet()
john.haveBirthday()