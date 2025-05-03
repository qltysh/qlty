// Unique field access patterns

class UniqueFieldPatterns {
    var regularField: String = "regular"
    lazy var lazyField: String = {
        return "This is computed lazily"
    }()
    
    // Property observers
    var observedField: Int = 0 {
        willSet {
            print("About to set observedField to \(newValue)")
        }
        didSet {
            print("observedField changed from \(oldValue) to \(observedField)")
        }
    }
}