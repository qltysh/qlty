// Swift protocols (interfaces)

protocol Vehicle {
    var numberOfWheels: Int { get }
    var description: String { get }
    
    func makeNoise()
}

class Bicycle: Vehicle {
    var numberOfWheels: Int = 2
    
    var description: String {
        return "Bicycle with \(numberOfWheels) wheels"
    }
    
    func makeNoise() {
        print("Ring ring!")
    }
}

class Car: Vehicle {
    var numberOfWheels: Int = 4
    
    var description: String {
        return "Car with \(numberOfWheels) wheels"
    }
    
    func makeNoise() {
        print("Vroom vroom!")
    }
}