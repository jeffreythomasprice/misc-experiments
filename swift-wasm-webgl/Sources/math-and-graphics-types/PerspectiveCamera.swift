import JavaScriptKit

class PerspectiveCamera<T: TypedArrayElement & StaticSized & FloatingPoint & Mathable & Sqrt & Trigonometry & AbsoluteValue> {
    // when the angles are both 0, points into the camera along the midpoint of the screen
    private let defaultForward: Vector3<T>
    // points toawrds the top of the screen
    private let defaultUp: Vector3<T>

    /*
	The actual orientation is a pair of rotations, in this order:
	- Rotate around the local Y axis, i.e. turn the camera left or right. Positive values turn right.
	- Rotate around the new local X axis, i.e. turn the camera up or down. Positive values are up.
	*/
    private var angleRight: Radians<T>
    private var angleUp: Radians<T>

    private var position: Vector3<T>

    init(
        position: Vector3<T>,
        target: Vector3<T>,
        up: Vector3<T>
    ) {
        print("TODO position = \(position)")
        print("TODO target = \(target)")
        print("TODO up = \(up)")

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
        print("TODO defaultForward = \(defaultForward)")

        defaultUp = up.normalized
        print("TODO defaultUp = \(defaultUp)")

        let q = Plane(normal: defaultUp, point: position).closestPointTo(point: target)
        print("TODO q = \(q)")
        let u = q - position
        print("TODO u = \(u)")
        let angleRight = Vector3.angleBetween(defaultForward, u)
        print("TODO angleRight = \(angleRight.degrees)")
        // TODO adjust angleRight based on whether target is to the left or right

        let g = target - position
        let angleUp = Vector3.angleBetween(g, u)
        print("TODO angleUp = \(angleUp.degrees)")
        // TODO adjust angleUp based on whether target is up or down

        self.angleRight = angleRight
        self.angleUp = angleUp

        self.position = position
    }

    // TODO accessors

    // TODO helper for moving

    // TODO helper for turning

    // TODO projection matrix

    // TODO modelview matrix
}
