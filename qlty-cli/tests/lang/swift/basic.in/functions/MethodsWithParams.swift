// Methods with parameters

class MethodsWithParams {
    private var bar: String
    private var baz: String
    
    init() {
        self.bar = ""
        self.baz = ""
    }
    
    func doSomething(baz: String, bar: String) -> String {
        self.bar = bar
        self.baz = baz
        return self.bar + self.baz
    }
    
    // Method with default parameter values
    func doSomethingElse(baz: String = "default baz", bar: String = "default bar") -> String {
        return baz + bar
    }
    
    // Method with variadic parameters
    func concatenate(strings: String...) -> String {
        return strings.joined()
    }
}