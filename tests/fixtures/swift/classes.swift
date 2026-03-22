import Foundation

class Calculator {
    private var value: Int = 0

    func add(a: Int, b: Int) -> Int {
        value = a + b
        return value
    }

    func print(message: String) {
        print(message)
    }
}

struct Point {
    let x: Double
    let y: Double
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
}
