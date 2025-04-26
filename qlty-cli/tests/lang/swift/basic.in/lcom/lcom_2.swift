// lcom=2

class KlassA {
    private var aaa: String = "aaa"
    private var bbb: String = "bbb"
    
    func getBbb() -> String {
        return bbb
    }
    
    func getAaa() -> String {
        return aaa
    }
    
    func foo() -> String {
        return getAaa()
    }
    
    func bar() -> String {
        return getBbb()
    }
}

class KlassB {
    private var baz: String = "baz"
    
    func getBar() -> String {
        return baz
    }
    
    func foo() -> String {
        return getBar()
    }
    
    func bar() -> String {
        return getBar()
    }
}