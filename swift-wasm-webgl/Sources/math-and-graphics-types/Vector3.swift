import JavaScriptKit

struct Vector3<T: TypedArrayElement & StaticSized> {
    let x: T
    let y: T
    let z: T
}

extension Vector3: TypedArraySerialization & StaticSized {
    static var lengthInBytes: Int { T.lengthInBytes * 3 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        destination[offset] = x
        offset += 1
        destination[offset] = y
        offset += 1
        destination[offset] = z
        offset += 1
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (Vector3<T>, Int) {
        var offset = offset
        let x = source[offset]
        offset += 1
        let y = source[offset]
        offset += 1
        let z = source[offset]
        offset += 1
        return (Vector3(x: x, y: y, z: z), offset)
    }
}

extension Vector3 where T: Mathable {
    static func + (left: Self, right: Self) -> Self {
        Self(
            x: left.x + right.x,
            y: left.y + right.y,
            z: left.z + right.z
        )
    }

    static func - (left: Self, right: Self) -> Self {
        Self(
            x: left.x - right.x,
            y: left.y - right.y,
            z: left.z - right.z
        )
    }

    static func * (left: Self, right: T) -> Self {
        Self(
            x: left.x * right,
            y: left.y * right,
            z: left.z * right
        )
    }

    static func * (left: T, right: Self) -> Self {
        Self(
            x: left * right.x,
            y: left * right.y,
            z: left * right.z
        )
    }

    static func / (left: Self, right: T) -> Self {
        Self(
            x: left.x / right,
            y: left.y / right,
            z: left.z / right
        )
    }

    static func dot(_ left: Self, _ right: Self) -> T {
        left.x * right.x + left.y * right.y + left.z * right.z
    }

    static func cross(_ left: Self, _ right: Self) -> Self {
        Self(
            x: left.y * right.z - left.z * right.y,
            y: left.z * right.x - left.x * right.z,
            z: left.x * right.y - left.y * right.x
        )
    }

    var magnitudeSquared: T { x * x + y * y + z * z }
}

extension Vector3 where T: FloatingPoint {
    static prefix func + (unary: Self) -> Self {
        unary
    }

    static prefix func - (unary: Self) -> Self {
        Self(
            x: -unary.x,
            y: -unary.y,
            z: -unary.z
        )
    }
}

extension Vector3 where T: Mathable & Sqrt {
    var magnitude: T { magnitudeSquared.sqrt }

    var normalized: Self { self / magnitude }
}
