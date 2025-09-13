struct Vector2<T> {
    let x: T
    let y: T
}

extension Vector2 where T: Mathable {
    static func + (left: Self, right: Self) -> Self {
        Self(
            x: left.x + right.x,
            y: left.y + right.y
        )
    }

    static func - (left: Self, right: Self) -> Self {
        Self(
            x: left.x - right.x,
            y: left.y - right.y
        )
    }

    static func * (left: Self, right: T) -> Self {
        Self(
            x: left.x * right,
            y: left.y * right
        )
    }

    static func * (left: T, right: Self) -> Self {
        Self(
            x: left * right.x,
            y: left * right.y
        )
    }

    static func / (left: Self, right: T) -> Self {
        Self(
            x: left.x / right,
            y: left.y / right
        )
    }

    static prefix func + (unary: Self) -> Self {
        unary
    }

    static prefix func - (unary: Self) -> Self {
        Self(
            x: -unary.x,
            y: -unary.y
        )
    }

    static func dot(_ left: Self, _ right: Self) -> T {
        left.x * right.x + left.y * right.y
    }

    var magnitudeSquared: T { x * x + y * y }
}

extension Vector2 where T: Mathable & Sqrt {
    var magnitude: T { magnitudeSquared.sqrt }

    var normalized: Self { self / magnitude }
}
