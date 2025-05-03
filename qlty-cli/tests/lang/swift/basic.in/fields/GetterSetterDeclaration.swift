// Class with getters and setters

class GetterSetterClass {
    private var _value: Int = 0
    
    // Computed property with getter and setter
    var value: Int {
        get {
            return _value
        }
        set {
            _value = newValue
        }
    }
    
    // Computed property with just a getter
    var doubleValue: Int {
        return _value * 2
    }
    
    // Property with a custom setter
    var tripleValue: Int {
        get {
            return _value * 3
        }
        set(newTripleValue) {
            _value = newTripleValue / 3
        }
    }
}