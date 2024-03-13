import JavaScriptKit

struct Plane<T: TypedArrayElement & StaticSized & Mathable & Sqrt> {
    let normal: Vector3<T>
    let d: T

    init(normal: Vector3<T>, point: Vector3<T>) {
        self.normal = normal.normalized
        d = Vector3.dot(normal, point)
    }

    /**
	Finds t such that (point + normal*t) is a point on the plane.
	*/
    func signedDistanceTo(point: Vector3<T>) -> T {
        d - Vector3.dot(normal, point)
    }

    func closestPointTo(point: Vector3<T>) -> Vector3<T> {
        normal * signedDistanceTo(point: point) + point
    }
}
