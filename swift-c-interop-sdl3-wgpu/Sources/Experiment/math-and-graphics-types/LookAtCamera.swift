class LookAtCamera<
    T: FloatingPoint & Mathable & Sqrt & Trigonometry
        & AbsoluteValue & TruncatingRemainderable
        & ExpressibleByFloatLiteral
> {
    // when the angles are both 0, points into the camera along the midpoint of the screen
    private let defaultForward: Vector3<T>
    // points toawrds the top of the screen
    private let defaultUp: Vector3<T>

    /*
	The actual orientation is a pair of rotations, in this order:
	- Rotate around the local Y axis, i.e. turn the camera left or right. Positive values turn right.
	- Rotate around the new local X axis, i.e. turn the camera up or down. Positive values are up.
	*/
    private var _angleRight: Radians<T>
    private var _angleUp: Radians<T>

    private var _position: Vector3<T>

    init(
        position: Vector3<T>,
        target: Vector3<T>,
        up: Vector3<T>
    ) {
        let forward = (target - position).normalized
        if forward.x.abs > forward.y.abs && forward.x.abs > forward.z.abs {
            if forward.x > 0 {
                defaultForward = Vector3(x: 1, y: 0, z: 0)
            } else {
                defaultForward = Vector3(x: -1, y: 0, z: 0)
            }
        } else if forward.y.abs > forward.z.abs {
            if forward.y > 0 {
                defaultForward = Vector3(x: 0, y: 1, z: 0)
            } else {
                defaultForward = Vector3(x: 0, y: -1, z: 0)
            }
        } else {
            if forward.z > 0 {
                defaultForward = Vector3(x: 0, y: 0, z: 1)
            } else {
                defaultForward = Vector3(x: 0, y: 0, z: -1)
            }
        }

        defaultUp = up.normalized

        let defaultRight = Vector3.cross(defaultUp, defaultForward)

        let q = Plane(normal: defaultUp, point: position).closestPointTo(point: target)
        let u = q - position
        let angleRight = Vector3.angleBetween(defaultForward, u)
        let angleRightFixed =
            if Vector3.dot(defaultRight, forward) > 0 {
                angleRight
            } else {
                Radians(T.pi * 2) - angleRight
            }
        self._angleRight = LookAtCamera.fixAngleRight(value: angleRightFixed)

        let g = target - position
        let angleUp = Vector3.angleBetween(g, u)
        let angleUpFixed =
            if Vector3.dot(defaultUp, forward) > 0 {
                angleUp
            } else {
                -angleUp
            }
        self._angleUp = LookAtCamera.fixAngleUp(value: angleUpFixed)

        self._position = position
    }

    var angleRight: Radians<T> {
        get { self._angleRight }
        set {
            self._angleRight = LookAtCamera.fixAngleRight(value: newValue)
        }
    }

    var angleUp: Radians<T> {
        get { self._angleUp }
        set {
            self._angleUp = LookAtCamera.fixAngleUp(value: newValue)
        }
    }

    var position: Vector3<T> {
        get { self._position }
        set { self._position = newValue }
    }

    func turn(mouseMovement: Vector2<T>) {
        // TODO put constants somewhere
        let v = mouseMovement / 700
        angleRight = angleRight + Degrees(45).radians * Radians(v.x)
        angleUp = angleUp + Degrees(45).radians * Radians(v.y)
    }

    func move(forward: T, strafe: T, up: T) {
        position =
            position
            + self.forward * forward
            + rightRightAngleOnly * strafe
            + defaultUp * up
    }

    var transformMatrix: Matrix4<T> {
        // TODO cache
        return Matrix4.lookAt(
            position: position,
            target: position + forward,
            up: defaultUp
        )
    }

    private var forward: Vector3<T> {
        // TODO cache
        return Matrix4.rotation(axis: rightRightAngleOnly, angle: angleUp).applyTo(
            vector: forwardRightAngleOnly)
    }

    private var rightRightAngleOnly: Vector3<T> {
        // TODO cache
        Vector3.cross(defaultUp, forwardRightAngleOnly)
    }

    private var forwardRightAngleOnly: Vector3<T> {
        // TODO cache
        Matrix4.rotation(axis: defaultUp, angle: angleRight).applyTo(vector: defaultForward)
    }

    private static func fixAngleRight(value: Radians<T>) -> Radians<T> {
        let x = value.truncatingRemainder(dividingBy: Radians(T.pi * 2))
        return if x < Radians(0) {
            // TODO put constants somewhere
            x + Radians(T.pi * 2)
        } else {
            x
        }
    }

    private static func fixAngleUp(value: Radians<T>) -> Radians<T> {
        // TODO put constants somewhere
        let limit = Radians(T.pi * 0.49)
        return clamp(value: value, min: -limit, max: limit)
    }
}
