// Static field declaration

class StaticFieldDeclaration {
    // Type properties (static fields)
    static var staticField: String = "This is a static field"
    
    // Class properties (can be overridden by subclasses)
    class var classField: String {
        return "This is a class field"
    }
    
    // Instance method accessing static field
    func accessStaticField() -> String {
        return StaticFieldDeclaration.staticField
    }
}