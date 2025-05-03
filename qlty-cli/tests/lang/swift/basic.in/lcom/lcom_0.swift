// lcom=0

class KlassA {
    private var field: String = "field"
    
    func method1() -> String {
        return field
    }
    
    func method2() -> String {
        return "prefix_" + field
    }
    
    func method3() -> String {
        return method1() + method2()
    }
}

class KlassB {
    private var fieldB: String = "fieldB"
    
    func getField() -> String {
        return fieldB
    }
    
    func setField(value: String) {
        fieldB = value
    }
}