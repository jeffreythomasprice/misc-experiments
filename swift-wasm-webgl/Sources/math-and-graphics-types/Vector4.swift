import JavaScriptKit

struct Vector4<T: TypedArrayElement & StaticSized> {
    let x: T
    let y: T
    let z: T
    let w: T
}

extension Vector4: TypedArraySerialization & StaticSized {
    static var lengthInBytes: Int { T.lengthInBytes * 4 }

    func writeTo(destination: JavaScriptKit.JSTypedArray<T>, offset: Int) -> Int {
        var offset = offset
        destination[offset] = x
        offset += 1
        destination[offset] = y
        offset += 1
        destination[offset] = z
        offset += 1
        destination[offset] = w
        offset += 1
        return offset
    }

    static func readFrom(source: JavaScriptKit.JSTypedArray<T>, offset: Int) -> (Vector4<T>, Int) {
        var offset = offset
        let x = source[offset]
        offset += 1
        let y = source[offset]
        offset += 1
        let z = source[offset]
        offset += 1
        let w = source[offset]
        offset += 1
        return (Vector4(x: x, y: y, z: z, w: w), offset)
    }
}

extension Vector4 where T: Mathable {
    static func + (left: Self, right: Self) -> Self {
        Self(
            x: left.x + right.x,
            y: left.y + right.y,
            z: left.z + right.z,
            w: left.w + right.w
        )
    }

    static func - (left: Self, right: Self) -> Self {
        Self(
            x: left.x - right.x,
            y: left.y - right.y,
            z: left.z - right.z,
            w: left.w - right.w
        )
    }

    static func * (left: Self, right: T) -> Self {
        Self(
            x: left.x * right,
            y: left.y * right,
            z: left.z * right,
            w: left.w * right
        )
    }

    static func * (left: T, right: Self) -> Self {
        Self(
            x: left * right.x,
            y: left * right.y,
            z: left * right.z,
            w: left * right.w
        )
    }

    static func / (left: Self, right: T) -> Self {
        Self(
            x: left.x / right,
            y: left.y / right,
            z: left.z / right,
            w: left.w / right
        )
    }

    static func dot(_ left: Self, _ right: Self) -> T {
        left.x * right.x + left.y * right.y + left.z * right.z + left.w * right.w
    }

    var magnitudeSquared: T { x * x + y * y + z * z + w * w }
}

extension Vector4 where T: Mathable & Sqrt {
    var magnitude: T { magnitudeSquared.sqrt }

    var normalized: Self { self / magnitude }
}
