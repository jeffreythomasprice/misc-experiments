import JavaScriptKit

struct Matrix4<T: TypedArrayElement & StaticSized> {
    let data: [T]
}

extension Matrix4 where T: ExpressibleByIntegerLiteral {
    static var identity: Self {
        Self(data: [
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ])
    }

    static func translation(_ v: Vector3<T>) -> Self {
        Self(data: [
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            v.x, v.y, v.z, 1,
        ])
    }

    static func scale(_ v: Vector3<T>) -> Self {
        Self(data: [
            v.x, 0, 0, 0,
            0, v.y, 0, 0,
            0, 0, v.z, 0,
            0, 0, 0, 1,
        ])
    }
}

extension Matrix4 where T: FloatingPoint & Mathable {
    static func * (left: Self, right: Self) -> Self {
        var result: [T] = [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]
        for i in 0..<4 {
            for j in 0..<4 {
                for k in 0..<4 {
                    result[i * 4 + j] += left.data[i * 4 + k] * right.data[k * 4 + j]
                }
            }
        }
        return Self(data: result)
    }

    func translate(_ v: Vector3<T>) -> Self {
        Self.translation(v) * self
    }

    func scale(_ v: Vector3<T>) -> Self {
        Self.scale(v) * self
    }

    static func ortho(
        left: T,
        right: T,
        bottom: T,
        top: T,
        near: T,
        far: T
    ) -> Self {
        Self(data: [
            2 / (right - left), 0, 0, -(right + left) / (right - left),
            0, 2 / (top - bottom), 0, -(top + bottom) / (top - bottom),
            0, 0, -2 / (far - near), -(far + near) / (far - near),
            0, 0, 0, 1,
        ])
    }
}

extension Matrix4 where T: FloatingPoint & Mathable & Sqrt & Trigonometry {
    static func rotation(axis: Vector3<T>, angle: Radians<T>) -> Self {
        let c = angle.cos
        let s = angle.sin
        let axis = axis.normalized
        return Self(data: [
            axis.x * axis.x * (1 - c) + c, axis.x * axis.y * (1 - c) - axis.z * s, axis.x * axis.z * (1 - c) + axis.y * s, 0,
            axis.y * axis.x * (1 - c) + axis.z * s, axis.y * axis.y * (1 - c) + c, axis.y * axis.z * (1 - c) - axis.x * s, 0,
            axis.x * axis.z * (1 - c) - axis.y * s, axis.y * axis.z * (1 - c) + axis.x * s, axis.z * axis.z * (1 - c) + c, 0,
            0, 0, 0, 1,
        ])
    }

    func rotate(axis: Vector3<T>, angle: Radians<T>) -> Self {
        Self.rotation(axis: axis, angle: angle) * self
    }

    static func perspective(
        verticalFieldOfView: Radians<T>,
        width: T,
        height: T,
        near: T,
        far: T
    ) -> Self {
        let f = 1 / (verticalFieldOfView / Radians(2)).tan
        let aspect = width / height
        return Self(data: [
            f / aspect, 0, 0, 0,
            0, f, 0, 0,
            0, 0, (far + near) / (near - far), 2 * far * near / (near - far),
            0, 0, -1, 0,
        ])
    }

    static func lookAt(
        position: Vector3<T>,
        target: Vector3<T>,
        up: Vector3<T>
    ) -> Self {
        let f = (target - position).normalized
        let up = up.normalized
        let s = Vector3.cross(f, up).normalized
        let u = Vector3.cross(s, f).normalized
        return Self(data: [
            s.x, u.x, -f.x, 0,
            s.y, u.y, -f.y, 0,
            s.z, u.z, -f.z, 0,
            0, 0, 0, 1,
        ]).translate(-position)
    }
}
