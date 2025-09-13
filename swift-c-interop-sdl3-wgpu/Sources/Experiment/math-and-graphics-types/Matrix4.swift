struct Matrix4<T> {
    let m00: T, m01: T, m02: T, m03: T
    let m10: T, m11: T, m12: T, m13: T
    let m20: T, m21: T, m22: T, m23: T
    let m30: T, m31: T, m32: T, m33: T
}

extension Matrix4 where T: ExpressibleByIntegerLiteral {
    static var identity: Self {
        Self(
            m00: 1, m01: 0, m02: 0, m03: 0,
            m10: 0, m11: 1, m12: 0, m13: 0,
            m20: 0, m21: 0, m22: 1, m23: 0,
            m30: 0, m31: 0, m32: 0, m33: 1
        )
    }

    static func translation(_ v: Vector3<T>) -> Self {
        Self(
            m00: 1, m01: 0, m02: 0, m03: 0,
            m10: 0, m11: 1, m12: 0, m13: 0,
            m20: 0, m21: 0, m22: 1, m23: 0,
            m30: v.x, m31: v.y, m32: v.z, m33: 1
        )
    }

    static func scale(_ v: Vector3<T>) -> Self {
        Self(
            m00: v.x, m01: 0, m02: 0, m03: 0,
            m10: 0, m11: v.y, m12: 0, m13: 0,
            m20: 0, m21: 0, m22: v.z, m23: 0,
            m30: 0, m31: 0, m32: 0, m33: 1
        )
    }
}

extension Matrix4 where T: FloatingPoint & Mathable {
    static func * (left: Self, right: Self) -> Self {
        func dot(_ row: (T, T, T, T), _ col: (T, T, T, T)) -> T {
            return row.0 * col.0 + row.1 * col.1 + row.2 * col.2 + row.3 * col.3
        }
        return Self(
            m00: dot(
                (left.m00, left.m01, left.m02, left.m03),
                (right.m00, right.m10, right.m20, right.m30)),
            m01: dot(
                (left.m00, left.m01, left.m02, left.m03),
                (right.m01, right.m11, right.m21, right.m31)),
            m02: dot(
                (left.m00, left.m01, left.m02, left.m03),
                (right.m02, right.m12, right.m22, right.m32)),
            m03: dot(
                (left.m00, left.m01, left.m02, left.m03),
                (right.m03, right.m13, right.m23, right.m33)),
            m10: dot(
                (left.m10, left.m11, left.m12, left.m13),
                (right.m00, right.m10, right.m20, right.m30)),
            m11: dot(
                (left.m10, left.m11, left.m12, left.m13),
                (right.m01, right.m11, right.m21, right.m31)),
            m12: dot(
                (left.m10, left.m11, left.m12, left.m13),
                (right.m02, right.m12, right.m22, right.m32)),
            m13: dot(
                (left.m10, left.m11, left.m12, left.m13),
                (right.m03, right.m13, right.m23, right.m33)),
            m20: dot(
                (left.m20, left.m21, left.m22, left.m23),
                (right.m00, right.m10, right.m20, right.m30)),
            m21: dot(
                (left.m20, left.m21, left.m22, left.m23),
                (right.m01, right.m11, right.m21, right.m31)),
            m22: dot(
                (left.m20, left.m21, left.m22, left.m23),
                (right.m02, right.m12, right.m22, right.m32)),
            m23: dot(
                (left.m20, left.m21, left.m22, left.m23),
                (right.m03, right.m13, right.m23, right.m33)),
            m30: dot(
                (left.m30, left.m31, left.m32, left.m33),
                (right.m00, right.m10, right.m20, right.m30)),
            m31: dot(
                (left.m30, left.m31, left.m32, left.m33),
                (right.m01, right.m11, right.m21, right.m31)),
            m32: dot(
                (left.m30, left.m31, left.m32, left.m33),
                (right.m02, right.m12, right.m22, right.m32)),
            m33: dot(
                (left.m30, left.m31, left.m32, left.m33),
                (right.m03, right.m13, right.m23, right.m33))
        )
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
        Self(
            m00: 2 / (right - left), m01: 0, m02: 0, m03: -(right + left) / (right - left),
            m10: 0, m11: 2 / (top - bottom), m12: 0, m13: -(top + bottom) / (top - bottom),
            m20: 0, m21: 0, m22: -2 / (far - near), m23: -(far + near) / (far - near),
            m30: 0, m31: 0, m32: 0, m33: 1
        )
    }
}

extension Matrix4 where T: FloatingPoint & Mathable & Sqrt & Trigonometry {
    static func rotation(axis: Vector3<T>, angle: Radians<T>) -> Self {
        let c = angle.cos
        let s = angle.sin
        let axis = axis.normalized
        return Self(
            m00: axis.x * axis.x * (1 - c) + c,
            m01: axis.x * axis.y * (1 - c) - axis.z * s,
            m02: axis.x * axis.z * (1 - c) + axis.y * s,
            m03: 0,
            m10: axis.y * axis.x * (1 - c) + axis.z * s,
            m11: axis.y * axis.y * (1 - c) + c,
            m12: axis.y * axis.z * (1 - c) - axis.x * s,
            m13: 0,
            m20: axis.x * axis.z * (1 - c) - axis.y * s,
            m21: axis.y * axis.z * (1 - c) + axis.x * s,
            m22: axis.z * axis.z * (1 - c) + c,
            m23: 0,
            m30: 0, m31: 0, m32: 0, m33: 1
        )
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
        return Self(
            m00: f / aspect, m01: 0, m02: 0, m03: 0,
            m10: 0, m11: f, m12: 0, m13: 0,
            m20: 0, m21: 0, m22: (far + near) / (near - far), m23: 2 * far * near / (near - far),
            m30: 0, m31: 0, m32: -1, m33: 0
        )
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
        let mat = Self(
            m00: s.x, m01: u.x, m02: -f.x, m03: 0,
            m10: s.y, m11: u.y, m12: -f.y, m13: 0,
            m20: s.z, m21: u.z, m22: -f.z, m23: 0,
            m30: 0, m31: 0, m32: 0, m33: 1
        )
        return mat.translate(-position)
    }
}

extension Matrix4 where T: Mathable {
    func applyTo(vector: Vector3<T>) -> Vector3<T> {
        Vector3(
            x: m00 * vector.x + m10 * vector.y + m20 * vector.z,
            y: m01 * vector.x + m11 * vector.y + m21 * vector.z,
            z: m02 * vector.x + m12 * vector.y + m22 * vector.z
        )
    }

    func applyTo(point: Vector3<T>) -> Vector3<T> {
        Vector3(
            x: m00 * point.x + m10 * point.y + m20 * point.z + m30,
            y: m01 * point.x + m11 * point.y + m21 * point.z + m31,
            z: m02 * point.x + m12 * point.y + m22 * point.z + m32
        )
    }
}
