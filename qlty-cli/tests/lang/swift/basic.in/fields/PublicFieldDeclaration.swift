// Public field declaration

class PublicFieldDeclaration {
    // By default, properties are internal in Swift, explicitly marking as public
    public var publicField: String = "This is public"
    
    func accessPublicField() -> String {
        return publicField
    }
}