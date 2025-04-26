// Swift enums with associated values and methods

// Basic enum
enum Direction {
    case north
    case south
    case east
    case west
    
    func description() -> String {
        switch self {
        case .north:
            return "North"
        case .south:
            return "South"
        case .east:
            return "East"
        case .west:
            return "West"
        }
    }
}

// Enum with associated values
enum Barcode {
    case upc(Int, Int, Int, Int)
    case qrCode(String)
    
    func description() -> String {
        switch self {
        case .upc(let numberSystem, let manufacturer, let product, let check):
            return "UPC: \(numberSystem)-\(manufacturer)-\(product)-\(check)"
        case .qrCode(let productCode):
            return "QR code: \(productCode)"
        }
    }
}

// Enum with raw values
enum Planet: Int {
    case mercury = 1
    case venus
    case earth
    case mars
    case jupiter
    case saturn
    case uranus
    case neptune
    
    func description() -> String {
        return "Planet #\(self.rawValue)"
    }
}