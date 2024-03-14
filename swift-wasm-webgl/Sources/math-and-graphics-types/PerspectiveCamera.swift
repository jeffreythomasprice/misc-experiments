import JavaScriptKit

class PerspectiveCamera<
    T: TypedArrayElement & StaticSized & FloatingPoint & Mathable & Sqrt & Trigonometry & AbsoluteValue & TruncatingRemainderable
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

    private var position: Vector3<T>

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
        self._angleRight = PerspectiveCamera.fixAngleRight(value: angleRightFixed)

        let g = target - position
        let angleUp = Vector3.angleBetween(g, u)
        let angleUpFixed =
            if Vector3.dot(defaultUp, forward) > 0 {
                angleUp
            } else {
                -angleUp
            }
        self._angleUp = PerspectiveCamera.fixAngleUp(value: angleUpFixed)

        self.position = position
    }

    var angleRight: Radians<T> {
        get { self._angleRight }
        set {
            self._angleRight = PerspectiveCamera.fixAngleRight(value: newValue)
        }
    }

    var angleUp: Radians<T> {
        get { self._angleUp }
        set {
            self._angleUp = PerspectiveCamera.fixAngleUp(value: newValue)
        }
    }

    // TODO helper for moving

    // TODO helper for turning

    // TODO projection matrix

    // TODO modelview matrix

    private static func fixAngleRight(value: Radians<T>) -> Radians<T> {
        let x = value.truncatingRemainder(dividingBy: Radians(T.pi * 2))
        return if x < Radians(0) {
            x + Radians(T.pi * 2)
        } else {
            x
        }
    }

    private static func fixAngleUp(value: Radians<T>) -> Radians<T> {
        let limit = Radians(T.pi * 0.99)
        return clamp(value: value, min: -limit, max: limit)
    }
}
