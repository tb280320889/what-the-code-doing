import Foundation
import UIKit

class Calculator {
    private var value: Int = 0

    func add(a: Int, b: Int) -> Int {
        value = a + b
        return value
    }

    func printMessage(_ message: String) {
        print(message)
    }
}

struct Point {
    let x: Double
    let y: Double

    func distanceTo(_ other: Point) -> Double {
        let dx = x - other.x
        let dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }
}

protocol IProcessor {
    func process(data: String)
}

enum Color {
    case red
    case green
    case blue
}

typealias StringMap = Dictionary<String, String>

extension String {
    func trimmed() -> String {
        return self.trimmingCharacters(in: .whitespaces)
    }

    var isNullOrEmpty: Bool {
        return isEmpty
    }
}

func main() {
    let calc = Calculator()
    let result = calc.add(a: 5, b: 10)
    print(result)
}
