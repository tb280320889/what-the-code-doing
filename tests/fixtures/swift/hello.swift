import Foundation

class Calculator {
    func add(a: Int, b: Int) -> Int {
        return a + b
    }

    func print(message: String) {
        print(message)
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

struct Point {
    let x: Double
    let y: Double
}

typealias StringMap = Dictionary<String, String>
