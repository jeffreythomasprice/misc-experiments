import JavaScriptKit

protocol Mathable {
    static func + (left: Self, right: Self) -> Self
    static func - (left: Self, right: Self) -> Self
    static func * (left: Self, right: Self) -> Self
    static func / (left: Self, right: Self) -> Self
    static prefix func + (unary: Self) -> Self
    static prefix func - (unary: Self) -> Self
}

extension Int8: Mathable {}
extension Int16: Mathable {}
extension Int32: Mathable {}
extension Int64: Mathable {}
extension Float32: Mathable {}
extension Float64: Mathable {}

protocol TruncatingRemainderable {
    func truncatingRemainder(dividingBy: Self) -> Self
}

extension Float32: TruncatingRemainderable {}
extension Float64: TruncatingRemainderable {}

protocol Trigonometry {
    var cos: Self { get }
    var sin: Self { get }
    var tan: Self { get }
    var acos: Radians<Self> { get }
    var asin: Radians<Self> { get }
    var atan: Radians<Self> { get }
    static func atan2(y: Self, x: Self) -> Radians<Self>
}

extension Float32: Trigonometry {
    var cos: Float32 {
        Float32(JSObject.global.Math.cos(self).number!)
    }

    var sin: Float32 {
        Float32(JSObject.global.Math.sin(self).number!)
    }

    var tan: Float32 {
        Float32(JSObject.global.Math.tan(self).number!)
    }

    var acos: Radians<Float32> {
        Radians(Float32(JSObject.global.Math.acos(self).number!))
    }

    var asin: Radians<Float32> {
        Radians(Float32(JSObject.global.Math.asin(self).number!))
    }

    var atan: Radians<Float32> {
        Radians(Float32(JSObject.global.Math.atan(self).number!))
    }

    static func atan2(y: Float32, x: Float32) -> Radians<Float32> {
        Radians(Float32(JSObject.global.Math.atan2(y, x).number!))
    }
}

extension Float64: Trigonometry {
    var cos: Float64 {
        JSObject.global.Math.cos(self).number!
    }

    var sin: Float64 {
        JSObject.global.Math.sin(self).number!
    }

    var tan: Float64 {
        JSObject.global.Math.tan(self).number!
    }

    var acos: Radians<Float64> {
        Radians(JSObject.global.Math.acos(self).number!)
    }

    var asin: Radians<Float64> {
        Radians(JSObject.global.Math.asin(self).number!)
    }

    var atan: Radians<Float64> {
        Radians(JSObject.global.Math.atan(self).number!)
    }

    static func atan2(y: Float64, x: Float64) -> Radians<Float64> {
        Radians(Float64(JSObject.global.Math.atan2(y, x).number!))
    }
}

protocol Sqrt {
    var sqrt: Self { get }
}

extension Float32: Sqrt {
    var sqrt: Float32 {
        Float32(JSObject.global.Math.sqrt(self).number!)
    }
}

extension Float64: Sqrt {
    var sqrt: Float64 {
        JSObject.global.Math.sqrt(self).number!
    }
}

protocol AbsoluteValue {
    var abs: Self { get }
}

extension Float32: AbsoluteValue {
    var abs: Float32 {
        if self < 0 {
            return -self
        }
        return self
    }
}

extension Float64: AbsoluteValue {
    var abs: Float64 {
        if self < 0 {
            return -self
        }
        return self
    }
}

func clamp<T: Comparable>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
