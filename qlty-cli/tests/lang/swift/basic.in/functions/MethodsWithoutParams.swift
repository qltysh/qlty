// Methods without parameters

class MethodsWithoutParams {
    private var bar: String = "bar"
    private var baz: String = "baz"
    
    func getBar() -> String {
        return bar
    }
    
    func getBaz() -> String {
        return baz
    }
    
    func getCombined() -> String {
        return getBar() + getBaz()
    }
    
    func doNothing() {
        // No parameters, no return value
    }
}