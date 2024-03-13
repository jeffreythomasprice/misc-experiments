import JavaScriptKit

struct Radians<T> {
    let value: T

    init(_ value: T) {
        self.value = value
    }
}

extension Radians where T: FloatingPoint {
    var degrees: Degrees<T> {
        Degrees(value * 180 / T.pi)
    }
}

extension Radians: Mathable where T: Mathable {
    static func + (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(left.value + right.value)
    }

    static func - (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(left.value - right.value)
    }

    static func * (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(left.value * right.value)
    }

    static func / (left: Radians<T>, right: Radians<T>) -> Radians<T> {
        Radians<T>(left.value / right.value)
    }
}

extension Radians: TruncatingRemainderable where T: TruncatingRemainderable {
    func truncatingRemainder(dividingBy: Radians<T>) -> Radians<T> {
        Self(value.truncatingRemainder(dividingBy: dividingBy.value))
    }
}

extension Radians where T: Trigonometry {
    var cos: T { value.cos }
    var sin: T { value.sin }
    var tan: T { value.tan }
}

struct Degrees<T> {
    let value: T

    init(_ value: T) {
        self.value = value
    }
}

extension Degrees where T: FloatingPoint {
    var radians: Radians<T> {
        Radians(value * T.pi / 180)
    }
}

extension Degrees: Mathable where T: Mathable {
    static func + (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(left.value + right.value)
    }

    static func - (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(left.value - right.value)
    }

    static func * (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(left.value * right.value)
    }

    static func / (left: Degrees<T>, right: Degrees<T>) -> Degrees<T> {
        Degrees<T>(left.value / right.value)
    }
}

extension Degrees: TruncatingRemainderable where T: TruncatingRemainderable {
    func truncatingRemainder(dividingBy: Degrees<T>) -> Degrees<T> {
        Self(value.truncatingRemainder(dividingBy: dividingBy.value))
    }
}

extension Degrees where T: FloatingPoint & Trigonometry {
    var cos: T { radians.cos }
    var sin: T { radians.sin }
    var tan: T { radians.tan }
}
