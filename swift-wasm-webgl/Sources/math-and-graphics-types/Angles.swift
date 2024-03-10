import JavaScriptKit

struct Radians<T: TypedArrayElement> {
    let value: T
}

extension Radians where T: FloatingPoint {
    var degrees: Degrees<T> { Degrees(value: value * 180 / T.pi) }
}

extension Radians: Mathable where T: Mathable {
    static func + (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(value: left.value + right.value)
    }

    static func - (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(value: left.value - right.value)
    }

    static func * (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(value: left.value * right.value)
    }

    static func / (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(value: left.value / right.value)
    }
}

extension Radians where T: Trigonometry {
    var cos: T { value.cos }
    var sin: T { value.sin }
    var tan: T { value.tan }
}

struct Degrees<T: TypedArrayElement> {
    let value: T
}

extension Degrees where T: FloatingPoint {
    var radians: Radians<T> { Radians(value: value * T.pi / 180) }
}

extension Degrees: Mathable where T: Mathable {
    static func + (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(value: left.value + right.value)
    }

    static func - (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(value: left.value - right.value)
    }

    static func * (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(value: left.value * right.value)
    }

    static func / (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(value: left.value / right.value)
    }
}

extension Degrees where T: FloatingPoint & Trigonometry {
    var cos: T { radians.cos }
    var sin: T { radians.sin }
    var tan: T { radians.tan }
}
