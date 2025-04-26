// Class declaration with fields

class Foo {
    var bar: String
    var baz: String
    
    // Constructor
    init() {
        self.bar = ""
        self.baz = ""
    }
}

class ClassDeclaration {
    static func doSomething() -> String {
        let foo = Foo()
        foo.bar = "Hello"
        foo.baz = "World"
        return foo.bar + foo.baz
    }
    
    static func main() {
        print(doSomething())
    }
}