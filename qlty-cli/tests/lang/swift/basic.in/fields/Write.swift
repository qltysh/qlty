// Field write operations

class FieldWriting {
    var field: String = ""
    
    func setField(value: String) {
        field = value
    }
    
    func appendToField(value: String) {
        field += value
    }
}