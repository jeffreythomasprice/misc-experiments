struct Vector4<T> {
    let x: T
    let y: T
    let z: T
    let w: T
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

    static prefix func + (unary: Self) -> Self {
        unary
    }

    static prefix func - (unary: Self) -> Self {
        Self(
            x: -unary.x,
            y: -unary.y,
            z: -unary.z,
            w: -unary.w
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
