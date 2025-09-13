struct Vector3<T> {
    let x: T
    let y: T
    let z: T
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

extension Vector3 where T: Mathable & Sqrt {
    var magnitude: T { magnitudeSquared.sqrt }

    var normalized: Self { self / magnitude }
}

// TODO also do this for 2d and 4d vectors
extension Vector3 where T: Mathable & Sqrt & Trigonometry {
    static func angleBetween(_ a: Vector3<T>, _ b: Vector3<T>) -> Radians<T> {
        /*
        https://stackoverflow.com/a/16544330/9290998
        https://stackoverflow.com/a/67719217/9290998
        x = dot(a, b)
        y = dot(n, cross(a, b))
        angle = atan2(y, x)
        */
        return T.atan2(
            y: cross(a, b).magnitude,
            x: dot(a, b)
        )
    }
}
