import JavaScriptKit

protocol Mathable {
    static func + (left: Self, right: Self) -> Self
    static func - (left: Self, right: Self) -> Self
    static func * (left: Self, right: Self) -> Self
    static func / (left: Self, right: Self) -> Self
}

extension UInt8: Mathable {}
extension Int8: Mathable {}
extension UInt16: Mathable {}
extension Int16: Mathable {}
extension UInt32: Mathable {}
extension Int32: Mathable {}
extension UInt64: Mathable {}
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
}

extension Float64: Trigonometry {
    var cos: Double {
        JSObject.global.Math.cos(self).number!
    }

    var sin: Double {
        JSObject.global.Math.sin(self).number!
    }

    var tan: Double {
        JSObject.global.Math.tan(self).number!
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
