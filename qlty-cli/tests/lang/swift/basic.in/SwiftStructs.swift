// Swift structs (value types)

// Basic struct
struct Point {
    var x: Double
    var y: Double
    
    func distanceFromOrigin() -> Double {
        return sqrt(x*x + y*y)
    }
    
    // Mutating method can change the struct's properties
    mutating func moveBy(x deltaX: Double, y deltaY: Double) {
        x += deltaX
        y += deltaY
    }
}

// Struct with initializers
struct Rectangle {
    var width: Double
    var height: Double
    
    // Custom initializer
    init(width: Double, height: Double) {
        self.width = width
        self.height = height
    }
    
    // Convenience initializer for squares
    init(sideLength: Double) {
        self.init(width: sideLength, height: sideLength)
    }
    
    // Computed property
    var area: Double {
        return width * height
    }
    
    var perimeter: Double {
        return 2 * (width + height)
    }
}

// Demonstrating value semantics vs. reference semantics
func valueVsReferenceSemantics() {
    // Structs are value types
    var point1 = Point(x: 1.0, y: 2.0)
    var point2 = point1 // Creates a copy
    
    point2.x = 5.0
    
    print("Point1: (\(point1.x), \(point1.y))") // (1.0, 2.0)
    print("Point2: (\(point2.x), \(point2.y))") // (5.0, 2.0)
    
    // Classes are reference types
    class PointClass {
        var x: Double
        var y: Double
        
        init(x: Double, y: Double) {
            self.x = x
            self.y = y
        }
    }
    
    let pointClass1 = PointClass(x: 1.0, y: 2.0)
    let pointClass2 = pointClass1 // Creates a reference, not a copy
    
    pointClass2.x = 5.0
    
    print("PointClass1: (\(pointClass1.x), \(pointClass1.y))") // (5.0, 2.0)
    print("PointClass2: (\(pointClass2.x), \(pointClass2.y))") // (5.0, 2.0)
}