func simple() {
    print("This is a simple function")
}

func complex() {
    let bar = 42
    if bar > 0 {
        if bar > 10 {
            if bar < 20 {
                if bar % 2 == 0 {
                    if bar % 3 == 0 {
                        print("Nested!")
                    }
                }
            }
        }
    }
}