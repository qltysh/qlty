// Singleton class with methods that have parameters

class SingletonMethodsWithParams {
    // Singleton instance
    private static var instance: SingletonMethodsWithParams?
    
    // Private initializer to prevent direct instantiation
    private init() {}
    
    // Method to get the singleton instance
    static func shared() -> SingletonMethodsWithParams {
        if instance == nil {
            instance = SingletonMethodsWithParams()
        }
        return instance!
    }
    
    // Methods with parameters
    func processData(data: String) -> String {
        return "Processed: \(data)"
    }
    
    func calculateValue(x: Int, y: Int) -> Int {
        return x * y
    }
}