// Testing conditional assignment for cognitive complexity

func main() {
    var bar = 0
    bar = bar != 0 ? bar : 10
    
    var foo = 0
    foo = foo != 0 && foo != 10 ? foo : 10
}