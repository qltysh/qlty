class BooleanLogic {
    var foo: Int = 0
    var bar: Int = 0
    
    func f0() {
        let x = foo - bar + 1
    }
}

class BooleanLogic1 {
    var foo: Bool = false
    var bar: Bool = false
    var baz: Bool = false
    var qux: Bool = false
    var zoo: Bool = false
    
    func f1() {
        if foo && bar && baz && qux && zoo {
            return
        }
    }
}