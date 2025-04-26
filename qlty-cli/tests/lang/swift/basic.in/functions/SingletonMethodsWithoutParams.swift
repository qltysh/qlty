// Singleton class with methods that have no parameters

class SingletonMethodsWithoutParams {
    // Singleton instance
    private static var instance: SingletonMethodsWithoutParams?
    
    // Private initializer to prevent direct instantiation
    private init() {}
    
    // Method to get the singleton instance
    static func shared() -> SingletonMethodsWithoutParams {
        if instance == nil {
            instance = SingletonMethodsWithoutParams()
        }
        return instance!
    }
    
    // Methods without parameters
    func getCurrentTimestamp() -> Double {
        return Date().timeIntervalSince1970
    }
    
    func getRandomNumber() -> Int {
        return Int.random(in: 1...100)
    }
}