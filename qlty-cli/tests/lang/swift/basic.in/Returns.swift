class Returns {
    func f0() {
        // No return statement
    }
    
    func f1() {
        return
    }
    
    func f2() {
        if true {
            return
        } else {
            return
        }
    }
    
    func f3() {
        if true {
            return
        } else if true {
            return
        } else {
            return
        }
    }
    
    func f4() {
        if true {
            return
        } else if true {
            return
        } else if true {
            return
        } else {
            return
        }
    }
}