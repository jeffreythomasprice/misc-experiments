import JavaScriptKit

struct Radians<T: TypedArrayElement> {
    let value: T
}

extension Radians where T: FloatingPoint {
    var degrees: Degrees<T> { Degrees(value: value * 180 / T.pi) }
}

extension Radians where T: Trigonometry {
    var cos: T { value.cos }
    var sin: T { value.sin }
}

struct Degrees<T: TypedArrayElement> {
    let value: T
}

extension Degrees where T: FloatingPoint {
    var radians: Radians<T> { Radians(value: value * T.pi / 180) }
}

extension Degrees where T: FloatingPoint & Trigonometry {
    var cos: T { radians.cos }
    var sin: T { radians.sin }
}
