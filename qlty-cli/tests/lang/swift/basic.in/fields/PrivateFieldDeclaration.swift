// Private field declaration

class PrivateFieldDeclaration {
    private var privateField: String = "This is private"
    
    func accessPrivateField() -> String {
        return privateField
    }
    
    func modifyPrivateField(newValue: String) {
        privateField = newValue
    }
}